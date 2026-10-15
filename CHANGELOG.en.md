# A-side changelog

[中文](CHANGELOG.md)

## 1.7.0

- The spoofing page keeps only two fields (boot hash and boot patch level); the OS and vendor patch levels are no longer written from the panel, so what the device reports cannot drift from its real state.
- The vendor / boot patch levels we report now use the 8-digit form the spec requires, so chains minted by the server keybox no longer lose the day.
- Attestation requests now carry this client's version, which the B side uses to decide whether to mint with the dates this client declares.
- The daemons now take a single-instance lock: when the manager started `service.sh` more than once, each copy used to fork its own `daemon` / `daemon-injector` and the two killed each other's keymint shadow every few seconds, which made attestation work only intermittently (`exited with code 137` in the log).

## 1.6.9

- A rebuilt settings panel: four swipeable pages (Config / Scope / Local keys / Spoofing) with live keymint, keystore2, adbr and SOTER status along the top.
- Six languages: 简体中文, 繁體中文, English, 日本語, 한국어, Русский.
- App scope, the local keybox (official public keybox / built-in self-signed chain / your own XML) and the spoofing values are all managed in the panel; the official address and token come pre-filled.
- Whether SOTER works is decided on the device and remembered across reboots; adbr remote control is off by default.
- System properties are now written with the same implementation Magisk uses, so no tampering traces are left in the property area.
