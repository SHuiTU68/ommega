# B-side changelog

[中文](CHANGELOG-B.md)

## 1.7.4

- Task pickup interval dropped from seconds to 80 ms: no more fixed delay where a task waited for the real device to poll.
- The uplink is only bound when a VPN is up, so long polls are no longer dragged down by the tunnel (which made the relay restart itself).
