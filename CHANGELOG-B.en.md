# B-side module changelog

[中文](CHANGELOG-B.md)

## 1.6.9

- The settings panel speaks six languages: 简体中文, 繁體中文, English, 日本語, 한국어, Русский, with the wording cleaned up throughout.
- The status card now shows four rows: relay, adbrd, the keystore service and the SOTER service.
- The panel can open the server status page directly, and "About" carries the version, the language picker, the QQ group and the GitHub repository.
- The card that used to be called "remote control" is now "forwarding": it decides whether this device serves the signing/attestation tasks the server hands out. With it off the device neither contacts the server nor accepts tasks. The adbr row stays "allow adbr remote control" (off by default, tick it yourself).
- The remote-control channel now uses a debug process shipped with the module (no need to install adbr yourself), and only that one process is recognised.
- Flipping a switch only changes what is on screen; writing the file is up to "Save". Reopening the panel re-reads the configuration.
