# A-side module changelog

[中文](CHANGELOG.md)

## 1.7.10

- AIDL presence markers are read the way AOSP reads them (any non-zero value
  means present). A marker other than 1 is no longer treated as an unknown
  request and handed to the system keystore2, so an alias no longer splits into
  a private-key entry and a certificate entry.
- rsbinder now points at the patched copy in vendor via a plain path dependency;
  it used to be declared as a patch, but that line had been swallowed by a
  comment, so the patch never took effect.

## 1.7.9

- Fingerprint state queries and enrolment work again on some devices.
- The background check no longer restarts the fingerprint service over and
  over, which keeps the log quiet and draws less power.
