# vendor/adbrd —— 随模块发布的 adbrd（设备侧）

`adbrd` 是 [adbr](https://github.com/jiyin004-jpg/adbr) 的设备侧程序：设备主动拨出去
连我们的服务器，服务器侧就能跑命令、传文件、开 adb 隧道。A / B 两个模块都带上它，
由各自的 WebUI 开关控制要不要跑。

**只有 arm64-v8a 一份。** 上游就只给 Android arm64 构建，别的架构不打包，
开关写了也当没写（`adbr-daemon.sh` 里直接跳过）。

## 这一份是哪来的

| 项 | 值 |
| --- | --- |
| 来源 | `D:\adbr-repo\adbr`，`cargo build --release --bin adbrd --target aarch64-linux-android` |
| 构建时间 | 2026-10-12 |
| 大小 / sha256 | 836848 字节 / `997e44c7f002677b05d9758a6b5ce458b29663257a9ed8e5fa2bc63f36833cb2` |
| 编入的服务端 | `110.40.170.96:1443,186.244.245.169:1443`（`ADBR_SERVER` 默认值） |
| 编入的公钥 | `88c680546be925cc4f8179626edd87dac94ac752933624938fca864ca98973dd`（`ADBR_PUBKEY`，服务端那把私钥对应的公钥） |

公钥和服务器地址都是**编译期**定死的，运行时改不了（故意的：产物在别人手里，
能改就等于把通道送人）。要换就重编、更新这里的二进制和上面这行 sha256。

二进制里只有**公钥**，没有私钥 —— 提交进仓库是安全的。

## 换一份的时候

1. 重编 `adbrd`（`ADBR_PUBKEY=<公钥> cargo build --release --bin adbrd --target aarch64-linux-android`）
2. 覆盖 `arm64-v8a/adbrd`，更新本文件的 sha256 / 大小 / 服务端
3. `python -m pytest` 不需要；构建时 `build.py` 会为它生成 `.sha256`，安装时由 `verify.sh` 校验
