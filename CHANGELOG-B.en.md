# B-side module changelog

[中文](CHANGELOG-B.md)

## 1.6.8

- New: the module card now opens a settings panel, and the usual settings can be changed right there (server address, device id, device name, token, logging switches and levels, key-material switch and concurrency, outgoing interface, keep-alive), taking effect as soon as you save - no reboot needed.
- New: an "allow remote control" master switch in the panel. Turned off, this device no longer talks to the server and picks up no tasks at all.
- New: an "allow adbr remote control" switch in the panel (off by default). Turning it on opens a separate public-network debug channel for the times the phone is out of reach; turn it off again when you are done.
- Fixed "the app hangs when turning on fingerprint payment / paying": if the B side had not been allowed to "create key material for apps", the app would keep repeating "create key -> look it up -> delete it" and never finish (that is exactly what production showed this round — the B-side log said the op was disabled). It is allowed by default now, and devices installed before this version pick the setting up automatically on upgrade, no reinstall needed.
- Added explanations for two situations that used to mislead: when the B side is not allowed to create material, the server log now says outright that it is a configuration problem (instead of looking like "this machine does not have that key"); and when an app repeatedly deletes and recreates on its own, a line is recorded with what to do about it.
- Fixed a misjudgement in capability reporting: a single failed signature used to be taken as "this device cannot sign at all", which could drop a perfectly capable machine out of the signing path.
- Hotfix #4: when configuration keys added by a newer version are filled in automatically on upgrade, environment-specific tuning keys such as the network bind setting are no longer touched along the way (so an upgrade cannot quietly change network settings that were working fine).
- Android 17 compatibility hotfix: prefer the platform `libc++.so` in the launcher so an older vendor C++ runtime cannot make `liblog.so` fail on the missing `__hash_memory` symbol before the B-side relay starts.

- Fixed the `system error'' loop when a real app (WeChat) sets up its fingerprint: one setting never reached the device relay, so the key-material step was refused and the app kept deleting and recreating it. Fixed at the source; takes effect right after the update, no reinstall needed.
