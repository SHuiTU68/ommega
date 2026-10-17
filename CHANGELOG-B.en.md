# B-side module changelog

[中文](CHANGELOG-B.md)

## 1.7.1

- The relay reports its own version, so the status page and the A-side panel show which release a B-side device runs.
- The patch-date follow-up no longer fires before the first unlock: while the device has not been unlocked since boot, the request is served with the local values - no property writes, no keymint/keystore2 restart - so the lock screen cannot end up rejecting a correct password.
