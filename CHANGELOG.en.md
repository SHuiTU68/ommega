# A-side changelog

[中文](CHANGELOG.md)

## 1.7.2

- SOTER is now answered by the module's own software service, which replaces the vendor SOTER service outright: signing still works when the vendor one cannot start, and WeChat plus the engineering-mode SOTER status follow along.
- The panel switch is now literally "replace the system SOTER service": turning it off restores the vendor one, and the takeover happens automatically at boot.
- Fixed the SOTER host crashing in a loop with the takeover on; if the takeover cannot come up it falls back to the vendor service instead of leaving the phone broken.
- The engineering-mode Key status self-check (Attestation key) no longer reports failure.
- Fixed the SOTER service restarting in a loop on some devices, which made WeChat fingerprint fail.
