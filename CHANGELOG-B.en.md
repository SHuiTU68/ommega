# B-side changelog

[中文](CHANGELOG-B.md)

## 1.7.7

- Patch-level following no longer slows attestation down: the date is applied directly instead of restarting the key service.
- If the system asks for a key upgrade before signing, it is upgraded and the operation continues instead of failing.
- The "unlocked device required" constraint is now written into the certificate.
