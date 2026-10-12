//! resetprop —— 模块自带的系统属性工具。
//!
//! 为什么要自己带一个：开机那一下要改的属性里有 `ro.*`（`ro.boot.*`、`ro.secure`
//! 这些），普通的 `setprop` 走 property_service，改不动已经生效的 `ro.*`；而各家 root
//! 管理器自带的 `resetprop` 不是都有（KernelSU 要 `ksud` 代管，有的管理器压根不提供），
//! 依赖它等于把开机伪装交给别人。
//!
//! 怎么改的（不重写属性区那套 trie / 内存分配）：
//!
//! 1. `__system_property_find` 找到 `prop_info`（这几个入口 bionic 的 libc.so 里
//!    本来就导出着，property_service 自己也用；2026-10-13 在真机 libc.so 的 dynsym
//!    里确认过，版本标签 `LIBC_PLATFORM` 且是默认版本，dlsym 按普通名字就能拿到）；
//! 2. 属性区在我们这种进程里是**只读映射**的（bionic 只给 init 开着写），所以先按
//!    `/proc/self/maps` 找到 `prop_info` 落在那块 `/dev/__properties__/xxx` 映射上，
//!    自己用 O_RDWR 打开同一个文件、`MAP_FIXED` 把那段映射就地换成可写
//!    （2026-10-13 实测 `u:r:ksu:s0` 下这些文件都能读写打开，没有 AVC 拒绝）；
//! 3. 再调 `__system_property_update`，序列号、长度处理这些都交给 libc 自己。
//!
//! 名字还不存在时先用 `__system_property_set`（走 property_service，新建属性它认），
//! 不行再试 `__system_property_add`；两者都写不进去就如实报错 —— 绝不能让开机脚本
//! 以为写成功了。
//!
//! 用法（只做我们真正用得到的那几件事，跟 Magisk 的 resetprop 对齐）：
//!
//! ```text
//! resetprop NAME VALUE     设置（含 ro.*）
//! resetprop NAME           读取，打印一行
//! resetprop -n NAME VALUE  跳过 property_service（我们本来就走直写，等价）
//! ```
//!
//! 不支持的：`-d/--delete`、`-p`（persist 文件）、`-f`、`-w`、`-c`。真需要再加。

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uint, c_void};

/// AOSP `include/sys/system_properties.h` 里的两个上限。
const PROP_NAME_MAX: usize = 32;
const PROP_VALUE_MAX: usize = 92;

/// `struct prop_info` 是不透明结构，我们只拿指针来回传。
#[repr(C)]
struct PropInfo {
    _opaque: [u8; 0],
}

type FindFn = unsafe extern "C" fn(*const c_char) -> *const PropInfo;
type UpdateFn = unsafe extern "C" fn(*mut PropInfo, *const c_char, c_uint) -> c_int;
type AddFn = unsafe extern "C" fn(*const c_char, c_uint, *const c_char, c_uint) -> c_int;
type SetFn = unsafe extern "C" fn(*const c_char, *const c_char) -> c_int;
type ReadCallbackFn = unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char, c_uint);
type ReadCallbackCaller = unsafe extern "C" fn(*const PropInfo, ReadCallbackFn, *mut c_void);

struct Api {
    find: FindFn,
    update: UpdateFn,
    add: AddFn,
    set: SetFn,
    read_callback: ReadCallbackCaller,
}

impl Api {
    /// 模块包的产物走**完全静态链接**（build.py 给这个 bin 单独加了
    /// `-C target-feature=+crt-static`）：静态二进制里没有动态链接器，dlsym 一条都
    /// 查不到，但 NDK 的 libc.a 里那几个实现本来就在（`llvm-nm libc.a` 查得到），
    /// 直接引用即可。
    #[cfg(target_feature = "crt-static")]
    fn load() -> Result<Self, String> {
        unsafe extern "C" {
            fn __system_property_find(name: *const c_char) -> *const PropInfo;
            fn __system_property_update(
                pi: *mut PropInfo,
                value: *const c_char,
                len: c_uint,
            ) -> c_int;
            fn __system_property_add(
                name: *const c_char,
                name_len: c_uint,
                value: *const c_char,
                value_len: c_uint,
            ) -> c_int;
            fn __system_property_set(name: *const c_char, value: *const c_char) -> c_int;
            fn __system_property_read_callback(
                pi: *const PropInfo,
                callback: ReadCallbackFn,
                cookie: *mut c_void,
            );
        }

        Ok(Self {
            find: __system_property_find,
            update: __system_property_update,
            add: __system_property_add,
            set: __system_property_set,
            read_callback: __system_property_read_callback,
        })
    }

    /// 动态链接的构建（CI 的 `cargo test --no-run`、本地开发构建）走这里：
    /// 那几个符号在 bionic 的 libc.so 里是导出着的，dlsym 按普通名字就能拿到。
    #[cfg(not(target_feature = "crt-static"))]
    fn load() -> Result<Self, String> {
        let handle = unsafe { libc::dlopen(c"libc.so".as_ptr(), libc::RTLD_NOW) };
        if handle.is_null() {
            return Err("dlopen(libc.so) 失败".to_string());
        }
        let lookup = |name: &str| -> Result<*mut c_void, String> {
            let cname = CString::new(name).map_err(|e| e.to_string())?;
            let symbol = unsafe { libc::dlsym(handle, cname.as_ptr()) };
            if symbol.is_null() {
                Err(format!("dlsym {name} 失败（这台机器的 libc 没有导出它）"))
            } else {
                Ok(symbol)
            }
        };

        Ok(Self {
            find: unsafe {
                std::mem::transmute::<*mut c_void, FindFn>(lookup("__system_property_find")?)
            },
            update: unsafe {
                std::mem::transmute::<*mut c_void, UpdateFn>(lookup("__system_property_update")?)
            },
            add: unsafe {
                std::mem::transmute::<*mut c_void, AddFn>(lookup("__system_property_add")?)
            },
            set: unsafe {
                std::mem::transmute::<*mut c_void, SetFn>(lookup("__system_property_set")?)
            },
            read_callback: unsafe {
                std::mem::transmute::<*mut c_void, ReadCallbackCaller>(lookup(
                    "__system_property_read_callback",
                )?)
            },
        })
    }

    fn find(&self, name: &str) -> Option<*const PropInfo> {
        let cname = CString::new(name).ok()?;
        let pi = unsafe { (self.find)(cname.as_ptr()) };
        if pi.is_null() {
            None
        } else {
            Some(pi)
        }
    }

    fn get(&self, name: &str) -> Option<String> {
        let pi = self.find(name)?;

        unsafe extern "C" fn collect(
            cookie: *mut c_void,
            _name: *const c_char,
            value: *const c_char,
            _serial: c_uint,
        ) {
            let out = &mut *(cookie as *mut Option<String>);
            if !value.is_null() {
                *out = Some(CStr::from_ptr(value).to_string_lossy().into_owned());
            }
        }

        let mut out: Option<String> = None;
        unsafe {
            (self.read_callback)(pi, collect, &mut out as *mut _ as *mut c_void);
        }
        out
    }

    /// 改一个已经存在的属性。返回 libc 的原始返回码，0 表示写成。
    fn update(&self, pi: *const PropInfo, value: &str) -> i32 {
        let Ok(cvalue) = CString::new(value) else {
            return -1;
        };
        unsafe { (self.update)(pi as *mut PropInfo, cvalue.as_ptr(), value.len() as c_uint) }
    }

    fn add(&self, name: &str, value: &str) -> i32 {
        let (Ok(cname), Ok(cvalue)) = (CString::new(name), CString::new(value)) else {
            return -1;
        };
        unsafe {
            (self.add)(
                cname.as_ptr(),
                name.len() as c_uint,
                cvalue.as_ptr(),
                value.len() as c_uint,
            )
        }
    }

    fn set_via_service(&self, name: &str, value: &str) -> i32 {
        let (Ok(cname), Ok(cvalue)) = (CString::new(name), CString::new(value)) else {
            return -1;
        };
        unsafe { (self.set)(cname.as_ptr(), cvalue.as_ptr()) }
    }
}

/// 属性区在普通进程里是只读映射的，改之前得先把这些映射换成可写。
///
/// 为什么是**全部**而不是只改 `pi` 所在那一块：`__system_property_update` 除了写
/// `prop_info` 自己的值，还要去碰它认定的「默认属性区」和全局 serial 区
/// （实测：只把 pi 那一块改可写时，值写进去了，然后写别的区就段错误）。
/// 这些区都是文件共享映射（`/dev/__properties__/<context>`），自己 O_RDWR 打开同一个
/// 文件、`MAP_FIXED` 就地覆盖即可 —— 文件是同一个，看到的还是同一份共享内存。
///
/// 2026-10-13 真机实测：`u:r:ksu:s0` 下这些文件都能读写打开，没有 AVC 拒绝。
fn make_property_areas_writable() -> Result<usize, String> {
    let maps = std::fs::read_to_string("/proc/self/maps")
        .map_err(|e| format!("读 /proc/self/maps 失败: {e}"))?;

    let mut patched = 0usize;
    for line in maps.lines() {
        let Some((range, rest)) = line.split_once(' ') else {
            continue;
        };
        let Some((start_hex, end_hex)) = range.split_once('-') else {
            continue;
        };
        let (Ok(start), Ok(end)) = (
            usize::from_str_radix(start_hex, 16),
            usize::from_str_radix(end_hex, 16),
        ) else {
            continue;
        };

        let fields: Vec<&str> = rest.split_whitespace().collect();
        if fields.len() < 5 {
            continue;
        }
        let perms = fields[0];
        let path = fields[4..].join(" "); // 跳过 perms/offset/dev/inode，剩下的是路径
        if !path.starts_with("/dev/__properties__/") {
            continue;
        }

        patched += 1;
        if perms.starts_with("rw") {
            continue; // 已经可写了
        }

        let cpath = CString::new(path.clone()).map_err(|e| e.to_string())?;
        let fd = unsafe { libc::open(cpath.as_ptr(), libc::O_RDWR | libc::O_CLOEXEC) };
        if fd < 0 {
            return Err(format!(
                "以读写方式打开 {path} 失败（SELinux 多半拒了）: {}",
                std::io::Error::last_os_error()
            ));
        }

        let mapped = unsafe {
            libc::mmap(
                start as *mut c_void,
                end - start,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED | libc::MAP_FIXED,
                fd,
                0,
            )
        };
        unsafe { libc::close(fd) };

        if mapped == libc::MAP_FAILED {
            return Err(format!(
                "{path} 改成可写映射失败: {}",
                std::io::Error::last_os_error()
            ));
        }
    }

    if patched == 0 {
        Err("没有找到任何 /dev/__properties__/ 映射".to_string())
    } else {
        Ok(patched)
    }
}

fn check_limits(name: &str, value: Option<&str>) -> Result<(), String> {
    if name.is_empty() || name.len() >= PROP_NAME_MAX {
        return Err(format!(
            "属性名太长或为空（上限 {} 字节）: {name:?}",
            PROP_NAME_MAX - 1
        ));
    }
    if let Some(value) = value {
        if value.len() >= PROP_VALUE_MAX {
            return Err(format!("属性值太长（上限 {} 字节）", PROP_VALUE_MAX - 1));
        }
    }
    Ok(())
}

fn do_set(api: &Api, name: &str, value: &str) -> Result<(), String> {
    check_limits(name, Some(value))?;

    if let Some(pi) = api.find(name) {
        make_property_areas_writable()?;
        let rc = api.update(pi, value);
        if rc != 0 {
            return Err(format!("__system_property_update 返回 {rc}"));
        }
    } else {
        // 名字还不存在：新建。先走 property_service（新建属性它认，也能顺便把它
        // 落到对应的 context 里），不行再直接往属性区里加一条。
        make_property_areas_writable()?;
        let mut rc = api.set_via_service(name, value);
        if api.get(name).as_deref() != Some(value) {
            rc = api.add(name, value);
        }
        if api.get(name).as_deref() != Some(value) {
            return Err(format!(
                "新建属性失败（property_service 返回 {rc}，直接写入也没生效）"
            ));
        }
    }

    // 读回来对着看：写没写进去以读到的为准，不靠返回值。
    match api.get(name) {
        Some(actual) if actual == value => Ok(()),
        Some(actual) => Err(format!(
            "写完了读回来是 {actual:?}，跟要写的 {value:?} 不一致"
        )),
        None => Err("写完了读不到这个属性".to_string()),
    }
}

fn usage() -> ! {
    eprintln!("用法: resetprop [-n] NAME [VALUE]");
    eprintln!("      resetprop NAME VALUE   设置（含 ro.*）");
    eprintln!("      resetprop NAME         读取，打印一行");
    std::process::exit(2);
}

fn main() {
    let mut args: Vec<String> = Vec::new();

    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            // 跳过 property_service —— 已有属性我们本来就走直写；收下这个选项只是
            // 为了跟 Magisk 的调用习惯兼容（脚本里可能带）。
            "-n" | "--skip-svc" => {}
            "-h" | "--help" => usage(),
            other if other.starts_with('-') && other.len() > 1 => {
                eprintln!("resetprop: 不支持的选项 {other}");
                std::process::exit(2);
            }
            other => args.push(other.to_string()),
        }
    }

    let (name, value) = match args.len() {
        1 => (args[0].as_str(), None),
        2 => (args[0].as_str(), Some(args[1].as_str())),
        _ => usage(),
    };

    if let Err(error) = check_limits(name, value) {
        eprintln!("resetprop: {error}");
        std::process::exit(2);
    }

    let api = match Api::load() {
        Ok(api) => api,
        Err(error) => {
            eprintln!("resetprop: {error}");
            std::process::exit(1);
        }
    };

    match value {
        Some(value) => {
            if let Err(error) = do_set(&api, name, value) {
                eprintln!("resetprop: 设置 {name} 失败: {error}");
                std::process::exit(1);
            }
        }
        None => match api.get(name) {
            Some(value) => println!("{value}"),
            None => {
                eprintln!("resetprop: {name} not found");
                std::process::exit(1);
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limits_match_aosp() {
        // 这两个数直接决定我们放不放行：写错就会把超长值交给 libc，
        // 报错信息变成 libc 的，反倒不好查。
        assert_eq!(PROP_NAME_MAX, 32);
        assert_eq!(PROP_VALUE_MAX, 92);
    }

    #[test]
    fn rejects_overlong_input_before_touching_libc() {
        assert!(check_limits("ro.x", Some("v")).is_ok());
        assert!(check_limits("", Some("v")).is_err());
        assert!(check_limits(&"a".repeat(PROP_NAME_MAX), Some("v")).is_err());
        assert!(check_limits("ro.x", Some(&"v".repeat(PROP_VALUE_MAX))).is_err());
    }
}
