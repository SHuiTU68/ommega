# A-side changelog

[中文](CHANGELOG.md)

## 1.7.0

- Device-ID attestation (the 710/711/712/716/717 group) is now carried only when the request is pushed to the server keybox by the B-side skip list; on the real-device hop the tags are stripped first. Not forwarding them at all made self-check apps report `hardware.attestation.device_properties` as an anomaly; forwarding them blindly made the real TEE reject the foreign IDs with `-66` and the chain fall back to the leaked keybox. The ID values now come from this device and are only honoured on the keybox path; callers that need a real TEE chain (GMS, Play) still get one without IDs.
- Source paths in client-visible error text are normalized to the AOSP-relative form: integrity checkers classify the backend by that string, and a Windows build path gives the relay away.
- When a caller asks for `INCLUDE_UNIQUE_ID` without holding the unique-ID permission (`REQUEST_UNIQUE_ID_ATTESTATION` / `keystore_key:gen_unique_id`), the whole request now fails with a permission error instead of silently dropping the tag. Dropping it made integrity checkers read the request as a permission bypass (TrustAttestor's `unique_id_permission`).
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
