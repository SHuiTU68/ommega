# B-side module changelog

[中文](CHANGELOG-B.md)

## 1.7.0

- New: mint with the patch dates the A side declares. When an attestation request carries dates, the relay writes them locally, has the keymint HAL re-read them, and only then mints the key; the switch `OMMEGA_RELAY_PATCHLEVEL_FOLLOW_A` is off by default.
- Older clients (A-side below 1.7.0) are left alone: a request without a version is served with the local values as-is, touching no property and restarting nothing.
- New skip list `OMMEGA_RELAY_PATCHLEVEL_SKIP_PKGS`: the packages listed there are always served with the local values.

## 1.6.9

- A settings panel in six languages: the status card shows relay, adbrd, the keystore service and the SOTER service, and opens the server status page in one tap.
- Forwarding, SOTER key material and adbr remote control, plus the server address, device id, logging and network settings, are all editable in the panel - press "Save" to apply.
- The debug process ships with the module; no separate adbr install is needed.
