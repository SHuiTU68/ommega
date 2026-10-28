# RKP keybox 签发（高通 / Duck ToolBox 路线）

给一台高通设备签一把 **私钥在我们手里、证书链到 Google 根** 的 `keybox.xml`。

和 `D:\repos\rkp-mtk-keybox`（MTK/Trustonic 的 importKey 路线）互补：那套要求设备 rkpd 库里已经有一把
Google 签过的 RKP 钥匙；这套走高通 TA 的 `CSR_V2` 私有口（命令号 `0x3122`），只要 TA 接受我们的公钥就行，
不依赖 rkpd 是否 provisioned。

## 原理

1. 引擎在设备上生成一对 P-256，**私钥只留在设备临时目录，跑完就拉回本地并删掉**；
2. 把我们的公钥塞进 `CSR_V2` 请求，TA 用它自己的 DICE 链给它签名；
3. 引擎拿这份合法的 CSR 去 `remoteprovisioning.googleapis.com` 换证书链（`fetchEekChain` + `signCertificates`）；
4. 叶子 + RKP 链（Droid CA3 → Droid CA2 → Key Attestation CA1）拼成 `keybox.xml`；
5. 本地再验一遍：每跳签名、私钥 ↔ 叶子公钥。

## 前置

- 设备：**高通 SoC**、已 root、`/dev/smcinvoke` 与 `/vendor/lib64/libkeymasterdeviceutils.so` 在位、
  TA 接受外部公钥（`rkp-device status` 里 `prerequisites_met=true` 且 `limitations` 只含
  `partial_device_support` / `rkp_only`）。实测 `SM8650`（zorn / 24117RK2CC，Android 16）可以。
- 引擎：`duckd`（旧名 `rkp-engine`），**必须是 2026 之后从 `D:\tools\Duck-ToolBox` 重编的**。
  2025 年那版自带过严的 DICE 校验，会误报
  `CSR validation failed: DICE chain entry 1 mode is not CBOR bytes`。
- 设备要能出国：国行机连不上 `remoteprovisioning.googleapis.com`，用一个 HTTP 代理（本脚本 `-ProxyUrl`）。

### 编引擎

```powershell
cd D:\tools\Duck-ToolBox
$env:ANDROID_HOME="D:\android-sdk"; $env:ANDROID_SDK_ROOT=$env:ANDROID_HOME
$env:JAVA_HOME="C:\Program Files\Android\Android Studio\jbr"
$env:CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="D:\android-ndk-r27d\toolchains\llvm\prebuilt\windows-x86_64\bin\aarch64-linux-android26-clang.cmd"
cargo build --release --target aarch64-linux-android -p duckd
```

产物在 `target\aarch64-linux-android\release\duckd`。

## 用法

```powershell
# 1) 设备能自己下载引擎时（推荐：把 engine.gz 放到一个设备可达的静态目录）
.\rkp-qualcomm.ps1 -Device <序列号> `
  -EngineUrl http://<静态服务器>/rkp-engine.gz `
  -ProxyUrl http://<user>:<pass>@<proxy-host>:<port>

# 2) 设备 adb 通道正常时，直接推
.\rkp-qualcomm.ps1 -Device <序列号> -EnginePath D:\tools\Duck-ToolBox\target\aarch64-linux-android\release\duckd -ProxyUrl ...

# 出厂网络本来就通 Google 的机器
.\rkp-qualcomm.ps1 -Device <序列号> -EngineUrl ... -NoProxy
```

产物：`D:\keys\rkp-keyboxes\<model>-<serial>-rkp.xml` + 同名 `.key.pem`。

## 踩过的坑（都写进脚本了）

| 现象 | 真因 | 解法 |
| --- | --- | --- |
| `CSR validation failed: DICE chain entry 1 mode is not CBOR bytes` | 2025 版引擎自带过严校验 | 重编引擎（见上） |
| `failed to locate Duck ToolBox module root` | 引擎要求模块根 | `DUCK_TOOLBOX_ROOT` 指向一个含 `module.prop` + `bin/` 的假根，目录要 **shell 用户**建 |
| `KeyMint rejected the CSR request with status -40`（`INVALID_TAG`） | 这台 TA 不接受这套请求（firmware 差异） | 换机型；`-40` 不是脚本能绕的 |
| `adbr push` 报 `服务端没回话 10054` / `punch` 报 NAT 不兼容 | 设备 adbd 数据通道打不通 | 走 `-EngineUrl` 让设备自己用 `curl` 拉引擎 |
| `adb connect 127.0.0.1:<tunnel>` 一直 `unauthorized` | `adbr authorize` 写的是 adbr 自己的公钥，不是 adb 的 | 不要绕隧道；需要时把 `~/.android/adbkey.pub` 自己写进 `/data/misc/adb/adb_keys`（脚本会备份并还原） |
| 国行机 `fetchEekChain` 超时/444 | rkpd / Google 区域限制 | 挂 HTTP 代理（`-ProxyUrl`），只用 `HTTPS_PROXY` 给引擎进程，不装 VPN |

## 痕迹清理（脚本收尾自动做，失败也会做）

```
/data/local/tmp/rkp-engine, rkp-sign.sh, *.out, keybox.xml, duckroot, duckdata
/data/adb/duck-toolbox
/data/misc/adb/adb_keys        ← 只在我们新增过时从备份还原
service.adb.tcp.port           ← 还原成 -1
设备端后台进程                  ← 结束后自然退出
```

全程只用 `adbr shell` / 文件读写，**不开投屏、不点授权框**，屏幕无任何变化。
