# B 端模块更新日志

[English](CHANGELOG-B.en.md)

## 1.7.0

- 新增「按 A 端声明的补丁日期出链」：收到带日期的认证请求时，先把日期落到本机、让 keymint 重读，再铸钥匙；开关 `OMMEGA_RELAY_PATCHLEVEL_FOLLOW_A` 默认关。
- 旧客户端（1.7.0 之前的 A 端）不参与：不带版本号的请求只按本机当前值处理，不碰属性、不重启服务。
- 新增黑名单 `OMMEGA_RELAY_PATCHLEVEL_SKIP_PKGS`，名单里的包同样只按本机当前值出链。

## 1.6.9

- 配置面板支持六种语言，状态卡显示 relay、adbrd、键库、SOTER 四项，并可一键打开服务端状态页。
- 转发、SOTER 建料、adbr 远控等开关，以及服务器地址、设备 ID、日志、网络等设置都在面板里改，点「保存」生效。
- 面板自带调试进程，不用另外装 adbr。
