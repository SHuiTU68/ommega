# A 端更新日志

[English](CHANGELOG.en.md)

## 1.7.0

- 调用方没有唯一 ID 权限（`REQUEST_UNIQUE_ID_ATTESTATION` / `keystore_key:gen_unique_id`）却带 `INCLUDE_UNIQUE_ID` 时，改成整笔回权限错误，不再把这个 tag 摘掉让请求成功。以前那套会被完整性检测读成「权限被绕过」（TrustAttestor 的 `unique_id_permission`）。
- 伪装参数页只留「哈希」和「启动补丁」两项：系统补丁与 vendor 补丁不再由面板改写，避免上报的日期跟机器实际状态对不上。
- 上报的 vendor / boot 补丁级别改成规范的 8 位格式，走服务器密钥库出的链不再缺那个「日」。
- 认证请求会带上本端版本号，B 端据此决定是否按本端声明的日期出链。
- 守护脚本加了单实例锁：以前管理器把 `service.sh` 拉起多份时会各起一套 `daemon` / `daemon-injector`，两边每轮都会清掉对方的 keymint 影子，表现为认证时好时坏（日志里一串 `exited with code 137`）。

## 1.6.9

- 全新配置面板：配置 / 作用域 / 本地密钥 / 伪装参数四页可左右滑动，顶栏实时显示 keymint、keystore2、adbr、SOTER 状态。
- 六种语言：简体中文、繁體中文、English、日本語、한국어、Русский。
- 应用作用域、本地密钥库（官方公开库 / 内置自签 / 自己的 XML）、伪装参数都在面板里管理，官方地址与授权码已预填。
- SOTER 能否使用由本机判定，重启后仍然记得；adbr 远控默认关闭。
- 系统属性的写入方式换成与 Magisk 同一套实现，不再在属性区留下改动痕迹。
