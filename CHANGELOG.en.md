# A-side module changelog

[中文](CHANGELOG.md)

## 1.7.8

- Two places no longer diverge from the platform: a request whose optional argument is encoded differently, or an EC/RSA signature that asks for another digest, used to be handed to the system backend (so the key we created could not be read back and the signature would not verify). Both now follow the platform's own rules.
- The local keys page can hold a separate keybox for the StrongBox level; without one both levels share the same file, and the leaf level always follows the request.
- New module icon.
- The spoof panel only syncs the boot hash and the boot patch level; nothing else
  is synced automatically.
- The spoof panel's manual editor covers every item (three patch levels, the
  system version, the five identity fields and the build fingerprint); clearing a
  field removes it from the overrides.
- The relay no longer drops the day from a patch level: a local chain stamped 2026-08-01 used to come back as 2026-08 over the relay. 
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
