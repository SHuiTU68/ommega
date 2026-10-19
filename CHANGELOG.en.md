# A-side changelog

[中文](CHANGELOG.md)

## 1.7.4

- Fingerprint payments no longer stall: the wait for the real device to pick up a task dropped from seconds to well under a hundred milliseconds.
- Detector apps no longer occupy the real device; their requests are handled locally, leaving the real device for apps that actually need fingerprint payments.
- Each step of a fingerprint payment dropped from about a second to the hundred-millisecond range, and the intermittent "setup failed" is gone.
