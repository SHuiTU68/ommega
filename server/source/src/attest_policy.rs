//! 认证层序的两个策略开关（运行时可变 + 落盘）。
//!
//! 这两个开关原来只读环境变量（`RELAY_ATTEST_KEYBOX_FIRST` /
//! `RELAY_ID_ATTEST_SERVER_SIDE`）：控制台上看不到、改完必须重启进程。现在改成
//! 运行时可变，控制台一保存就立即生效，同时写进 `data/attest_policy.json`，
//! 重启后仍然有效。环境变量退化成「文件里没有值时」的初始值，老部署行为不变。
//!
//! - `keybox_first`：**所有** attest 的层序。`false`（默认）= 真机优先
//!   `b → keybox → self_signed`；`true` = 服务端 keybox 优先。
//! - `id_attest_server_side`：带 `attestation_ids`（设备 ID 认证）的 attest
//!   是否也走服务端 keybox 优先。`false`（默认）= 同样先问真机。
//!
//! 注意别和 `strongbox::mode()` 搞混：那个只管 StrongBox 请求的编排，跟这里无关。

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Once;

/// 落盘位置。跟其它状态文件一样按进程工作目录算（systemd 的 WorkingDirectory=/opt/relay）。
pub const PATH: &str = "data/attest_policy.json";

static KEYBOX_FIRST: AtomicBool = AtomicBool::new(false);
static ID_ATTEST_SERVER_SIDE: AtomicBool = AtomicBool::new(false);
static LOADED: Once = Once::new();

fn env_flag(key: &str) -> bool {
    matches!(
        std::env::var(key)
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    )
}

/// 启动时调一次：先用环境变量当初始值（兼容老部署），再用落盘值覆盖。
pub fn init() {
    LOADED.call_once(|| {
        KEYBOX_FIRST.store(env_flag("RELAY_ATTEST_KEYBOX_FIRST"), Ordering::Relaxed);
        ID_ATTEST_SERVER_SIDE.store(env_flag("RELAY_ID_ATTEST_SERVER_SIDE"), Ordering::Relaxed);
        load_file();
    });
}

fn load_file() {
    let Ok(text) = std::fs::read_to_string(PATH) else {
        return;
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
        // 文件写坏了就当没有：别让一个坏 JSON 把服务拖不起来。
        return;
    };
    if let Some(b) = v.get("keybox_first").and_then(|x| x.as_bool()) {
        KEYBOX_FIRST.store(b, Ordering::Relaxed);
    }
    if let Some(b) = v.get("id_attest_server_side").and_then(|x| x.as_bool()) {
        ID_ATTEST_SERVER_SIDE.store(b, Ordering::Relaxed);
    }
}

/// 所有 attest 是否「服务端 keybox 优先」。
pub fn keybox_first() -> bool {
    init();
    KEYBOX_FIRST.load(Ordering::Relaxed)
}

/// 带设备 ID 的 attest 是否「服务端 keybox 优先」。
pub fn id_attest_server_side() -> bool {
    init();
    ID_ATTEST_SERVER_SIDE.load(Ordering::Relaxed)
}

/// 当前两个开关的值。
pub fn snapshot() -> (bool, bool) {
    init();
    (keybox_first(), id_attest_server_side())
}

/// 改开关并落盘。`None` = 这个字段不动。
pub fn set(keybox_first: Option<bool>, id_attest_server_side: Option<bool>) -> std::io::Result<()> {
    init();
    if let Some(b) = keybox_first {
        KEYBOX_FIRST.store(b, Ordering::Relaxed);
    }
    if let Some(b) = id_attest_server_side {
        ID_ATTEST_SERVER_SIDE.store(b, Ordering::Relaxed);
    }
    let (k, i) = snapshot();
    let body = serde_json::json!({
        "keybox_first": k,
        "id_attest_server_side": i,
    });
    if let Some(dir) = Path::new(PATH).parent() {
        std::fs::create_dir_all(dir)?;
    }
    let text = serde_json::to_string_pretty(&body).unwrap_or_default();
    let tmp = format!("{PATH}.tmp");
    std::fs::write(&tmp, text.as_bytes())?;
    std::fs::rename(&tmp, PATH)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_flag_parsing() {
        assert!(env_flag("__OMMEGA_NEVER_SET_THIS__") == false);
    }

    #[test]
    fn set_and_snapshot_roundtrip() {
        // 只验内存态；落盘路径依赖工作目录，单测里不去碰它。
        init();
        KEYBOX_FIRST.store(true, Ordering::Relaxed);
        ID_ATTEST_SERVER_SIDE.store(false, Ordering::Relaxed);
        assert_eq!(snapshot(), (true, false));
        KEYBOX_FIRST.store(false, Ordering::Relaxed);
        assert_eq!(snapshot(), (false, false));
    }
}
