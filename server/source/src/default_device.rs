//! 「默认设备 id」：A 端留空时服务端拿它挑 B。
//!
//! A 端远程模式默认不带 `device_id`（模块装好就是这样，面板也不用占位一台设备）。
//! 请求照样发上来，由服务端决定落到哪台 B：
//!
//!   1. 控制台里填了「默认设备 id」→ 就用那一台；
//!   2. 没填（或填的那台当前离线）→ 按负载挑一台在线的（`resolve_online_target`）。
//!
//! 为什么要落盘：A 端一装好就是留空，几乎所有设备都吃这个默认值。进程重启后如果
//! 退回「按负载挑」，同一轮 SOTER 流程的前后两笔就可能落到不同 B 上 —— 而同一个
//! 槽位的 ASK / AuthKey / 签名必须出自同一台（见 `soter_gate`）。所以运营在控制台
//! 填的那一台必须活过重启。
//!
//! 优先级：`RELAY_DEFAULT_DEVICE_ID` 环境变量（文件为空时当初始值）→
//! `data/default_device.json`。两者都没有就是空 = 按负载挑。

use std::path::Path;
use std::sync::RwLock;

use serde_json::Value;

/// 落盘位置。跟其它状态文件一样按进程工作目录算（systemd 的 WorkingDirectory=/opt/relay）。
/// `OMMEGA_DEFAULT_DEVICE_FILE` 可覆盖。
pub const DEFAULT_PATH: &str = "data/default_device.json";

/// 环境变量里的初始值（一次性）。
pub const ENV_KEY: &str = "RELAY_DEFAULT_DEVICE_ID";

/// 设备 id 的长度上限。`device-b-xxxxxxxx` 是 17 个字符，留足余量。
const MAX_LEN: usize = 128;

static DEVICE_ID: RwLock<String> = RwLock::new(String::new());

/// 路径：`OMMEGA_DEFAULT_DEVICE_FILE` 优先。
fn path() -> String {
    std::env::var("OMMEGA_DEFAULT_DEVICE_FILE")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| DEFAULT_PATH.to_string())
}

/// 当前的默认设备 id（空字符串 = 没指定，按负载挑）。
pub fn get() -> String {
    DEVICE_ID
        .read()
        .map(|v| v.clone())
        .unwrap_or_else(|poisoned| poisoned.into_inner().clone())
}

/// 规范化一个设备 id：去空白；空字符串表示「不指定」。
///
/// 只做宽松校验（不允许空白字符、长度上限），因为设备 id 是运营自己给 B 端起的，
/// 有自己的命名习惯；这里拦住的只是明显写错的东西（粘贴进来带空格/换行、超长串）。
fn normalize(raw: &str) -> Result<String, String> {
    let id = raw.trim();
    if id.len() > MAX_LEN {
        return Err(format!("device id too long (>{MAX_LEN} bytes)"));
    }
    if id.chars().any(|c| c.is_whitespace()) {
        return Err("device id must not contain whitespace".to_string());
    }
    Ok(id.to_string())
}

/// 设置默认设备 id。空字符串 = 清掉（回到按负载挑）。
pub fn set(raw: &str) -> Result<String, String> {
    let id = normalize(raw)?;
    persist(&id)?;
    if let Ok(mut w) = DEVICE_ID.write() {
        *w = id.clone();
    }
    Ok(id)
}

/// 启动时装载一次。环境变量只在文件里没有值时当初始值（环境变量写错了不该
/// 每次重启都把控制台保存的值盖掉）。
pub fn init() {
    let file_path = path();
    let from_file = read_file(&file_path);
    let id = if from_file.is_empty() {
        let env = std::env::var(ENV_KEY).unwrap_or_default();
        let env = normalize(&env).unwrap_or_default();
        if !env.is_empty() {
            // 把环境变量当成一次性的播种：写进文件，之后控制台改的就以文件为准。
            if let Err(e) = persist(&env) {
                tracing::warn!("default_device: 播种 {} 失败: {e}", file_path);
            }
        }
        env
    } else {
        from_file
    };
    if let Ok(mut w) = DEVICE_ID.write() {
        *w = id.clone();
    }
    if id.is_empty() {
        tracing::info!("default_device: 未指定，A 端留空时按负载挑一台在线 B");
    } else {
        tracing::info!("default_device: {id}（A 端留空时优先落这台）");
    }
}

fn read_file(file_path: &str) -> String {
    let Ok(text) = std::fs::read_to_string(file_path) else {
        return String::new();
    };
    // 文件格式就是 `{"device_id":"..."}`；解析失败就当没配，不要因此起不来。
    let v: serde_json::Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("default_device: {file_path} 解析失败（{e}），当作未指定");
            return String::new();
        }
    };
    normalize(v.get("device_id").and_then(|x| x.as_str()).unwrap_or("")).unwrap_or_default()
}

fn persist(id: &str) -> Result<(), String> {
    let file_path = path();
    if let Some(parent) = Path::new(&file_path).parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir {parent:?}: {e}"))?;
    }
    let body = serde_json::json!({ "device_id": id }).to_string();
    // tmp + rename：写一半崩掉不至于把上一份好配置换成半截 JSON。
    let tmp = format!("{file_path}.tmp");
    std::fs::write(&tmp, body.as_bytes()).map_err(|e| format!("write {tmp}: {e}"))?;
    std::fs::rename(&tmp, &file_path).map_err(|e| format!("rename {tmp} -> {file_path}: {e}"))?;
    Ok(())
}

/// 把一个 A 端请求的 `device_id` 补齐成「实际要用的那台」。
///
/// 语义就是用户定下的那条：A 端留空 + 服务端配了默认设备 = **完全等同于 A 端自带了
/// 那个 id**。所以这里在入口处就把 body 归一，而不是在某一层里临时拿默认值——归一之后，
/// physical / serverbox 两种模式的层序、StrongBox 分流、身份查找、队列派发、SOTER 的
/// 槽位钉/会话路由看见的都是同一台具体设备，行为与「A 端真填了这个 id」一字不差。
///
/// 没配默认值（或 body 里已经有 id）时原样返回，空 id 继续走 B 层的按负载挑选。
pub fn fill_in_body(body: &Value) -> Value {
    let raw = body
        .get("device_id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    if !raw.is_empty() {
        return body.clone();
    }
    let id = get();
    if id.is_empty() {
        return body.clone();
    }
    let mut out = body.clone();
    if let Some(obj) = out.as_object_mut() {
        obj.insert("device_id".to_string(), Value::String(id));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_trims_and_rejects_junk() {
        assert_eq!(normalize("  device-b-1  "), Ok("device-b-1".to_string()));
        assert_eq!(normalize(""), Ok(String::new()));
        assert!(normalize("device b-1").is_err());
        assert!(normalize(&"x".repeat(MAX_LEN + 1)).is_err());
    }

    #[test]
    fn set_and_get_roundtrip() {
        let dir = std::env::temp_dir().join(format!("ommega-dd-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("default_device.json");
        std::env::set_var("OMMEGA_DEFAULT_DEVICE_FILE", &file);

        assert_eq!(set("device-b-test").unwrap(), "device-b-test");
        assert_eq!(get(), "device-b-test");
        // 落盘了：重新读一遍也还是它。
        assert_eq!(read_file(file.to_str().unwrap()), "device-b-test");

        // 入口归一：空 id 被补上默认设备；已经带 id 的原样保留（哪怕它不在线）。
        let empty = serde_json::json!({ "challenge": "AA", "device_id": "" });
        assert_eq!(fill_in_body(&empty)["device_id"], "device-b-test");
        let missing = serde_json::json!({ "challenge": "AA" });
        assert_eq!(fill_in_body(&missing)["device_id"], "device-b-test");
        let named = serde_json::json!({ "device_id": "device-b-own" });
        assert_eq!(fill_in_body(&named)["device_id"], "device-b-own");

        // 清空 = 回到按负载挑：入口不再补任何东西。
        assert_eq!(set("").unwrap(), "");
        assert_eq!(get(), "");
        assert_eq!(read_file(file.to_str().unwrap()), "");
        assert_eq!(fill_in_body(&empty)["device_id"], "");

        let _ = std::fs::remove_dir_all(&dir);
        std::env::remove_var("OMMEGA_DEFAULT_DEVICE_FILE");
    }
}
