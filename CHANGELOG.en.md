# A-side changelog

[中文](CHANGELOG.md)

## 1.6.9

- The settings panel has been rebuilt: four pages (Config / Scope / Local keys / Spoofing) you swipe between, an "About" button in the top right, and a status strip along the top showing whether keymint, keystore2, adbr and SOTER are running.
- Six languages: 简体中文, 繁體中文, English, 日本語, 한국어, Русский.
- Scope: only the apps you installed are listed by default; press "Show system apps" to bring the system ones in too. Search, ticking and saving all happen in that one list, and "select all / clear" only applies to what is currently shown.
- Config: the official server address and token come pre-filled, so it works right after installing; the device id can stay empty - it picks an online, usable device by itself.
- Status: whether SOTER works is decided on the device - one successful signature is enough and it is remembered across reboots. The adbr remote-control debug process ships with the module and is off by default.
- Local keys: take the official public keybox, a built-in self-signed chain, or write your own XML file; SOTER keys can be written or cleared here too.
- Spoofing: system / boot / vendor patch dates and the boot image digest, shown one per line and applied as soon as you save. The property tool ships with the module, so no extra one from the root manager is needed.
