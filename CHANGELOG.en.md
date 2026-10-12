# A-side changelog

[中文](CHANGELOG.md)

## 1.6.9

- The panel was rebuilt: four pages (Config / Scope / Local keys / Spoofing) you swipe between, an "About" button in the top right, and a status strip along the top showing whether keymint, keystore2, adbr and SOTER are running.
- The panel speaks six languages: 简体中文, 繁體中文, English, 日本語, 한국어, Русский.
- The app list now shows only the apps you installed; press "Show system apps" to bring the system ones in too. Search, ticking and saving all happen in that one list.
- The server address and token come pre-filled with the official values, so it works right after installing; the device id can stay empty - it picks an online, usable device by itself.
- Whether SOTER works is now decided on the device: one successful signature (local, relay or server) is enough for the panel to show it as available, and it is remembered across reboots instead of asking the server again.
- The "current" values under Spoofing are shown one per line, and the pointless "boot public key" entry is gone.
- The property tool used by Spoofing now ships with the module instead of coming from the root manager - some managers (KernelSU, for instance) do not provide it, which used to leave those properties unwritten at boot.
- adbr remote control: the debug process now ships with the module (no need to install adbr yourself), and the switch is off by default. Turning it on opens a separate public-network debug channel for when the phone is out of reach.
- The keybox page dropped the "device built-in" entry and renamed "from file" to "custom".
