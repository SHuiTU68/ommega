# A-side changelog

[中文](CHANGELOG.md)

## 测试

- Leaving the device id empty no longer errors out and no longer falls back to the phone's own software chain: the request still goes out and the server decides which B device serves it — the "default device id" set in the admin console (`/jiyin004/`) first, otherwise the idlest online one. A fresh install leaves it empty; `device-b-2` is no longer used as a placeholder.
- The admin console gains a "default device id" setting you can fill in or clear.
- When the SOTER setup keeps failing (the app rebuilds and deletes its own keys because of a stale local cache), it is reset automatically once: the app is stopped, its SOTER cache is cleared, and that uid's material on the B device is wiped — no manual step needed.

## 1.7.1

- The panel and the status page show the version of the B-side device the phone is paired with.
- With the device id left empty, the panel prefers a B-side device whose boot patch falls in the same year as the phone, whose version is close, and whose other patch dates are close too; the phone-app B devices are never picked.
- On the status page, devices that pass the self-check are listed first and failures last.
- The panel pages follow your finger: one swipe turns one page and the tab highlight slides along; the tab labels are legible in dark mode.
- Turning off "Allow adbr remote control" now also closes the debug port it opened, with no reboot needed.
- Unticking every app in the scope really intercepts none - the built-in defaults no longer come back.
