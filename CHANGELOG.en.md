# A-side module changelog

[中文](CHANGELOG.md)

## 1.7.8

- The spoof panel gained a sync mode: mode 1 aligns the whole device with the
  selected B (model, brand, fingerprint, system version, the three patch levels
  and the boot hash), mode 2 only syncs the boot hash and the boot patch level.
  Mode 1 is the default.
- "Declare no StrongBox" now works on its own, without the root manager mounting
  the module: after a reboot the system reports no StrongBox, and the other
  features declared in the same file (app_attest_key, for example) stay intact.
- The local fallback SOTER key is labelled as ATTK in the panel; the custom file
  is the signing root.
- The server-side SOTER fallback no longer uses one key as both ASK and ATTK, so
  WeChat enrolment stops failing on the mismatch.
- Declaring no device ID attestation is off by default; taking over SOTER is on.
- Global scope is on by default: ordinary apps go through the module while system
  services and ROM components keep their original backend.
- Engineer mode key operations work out of the box.
- Saving the configuration from the panel no longer fails with "No such file or directory":
  a missing state symlink or bundled resetprop is now recreated automatically.
- The SOTER log no longer repeats the same resolution line every few seconds.
