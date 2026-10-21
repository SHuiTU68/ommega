# A-side changelog

[中文](CHANGELOG.md)

## 1.7.6

- The SOTER checks no longer fail: the device id is returned in the same form as on a real device, and the codes for a missing key or a missing session now match too.
- On older platforms that only expose the legacy interface, SOTER no longer keeps dropping out.
- The SOTER key row on the factory engineering-mode page passes as well.
- SOTER takeover also works on MediaTek (Trustonic) devices.
- Fixed adbr screen mirroring failing to connect on some devices.
- Fixed SOTER signing occasionally reporting errors.
- When the followed device is offline, another working one is picked instead of failing outright.
- The local fallback path now actually answers: it used to stay silent because the device serial could not be read.
- Logs live in one place and carry the process id, so a failure can be traced to the component that wrote it.
