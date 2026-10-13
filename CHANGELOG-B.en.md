# B-side module changelog

[中文](CHANGELOG-B.md)

## 1.6.9

- The settings panel speaks six languages: 简体中文, 繁體中文, English, 日本語, 한국어, Русский.
- Status: four rows - relay, adbrd, the keystore service and the SOTER service - each with a dot and a pid, plus a button that opens the server status page.
- Forwarding: this switch decides whether the device serves the signing / attestation tasks the server hands out. With it off the device neither contacts the server nor accepts tasks.
- SOTER key material: whether this device may create or delete SOTER key material (on by default). When off the device leaves the slot alone and forwarding is unaffected.
- adbr remote control (off by default): opens a separate public-network debug channel; the debug process ships with the module.
- Everything else is editable in the panel: server address, device id, device name, token, logging switches and levels, outgoing interface, keep-alive. Press "Save" to apply.
