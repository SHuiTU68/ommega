#!/system/bin/sh
# ommega-b module uninstall hook.
#
# Runs automatically when the module is removed via a root manager
# (KernelSU / Magisk / APatch). Kills the relay agent and removes the B-side
# files under the shared /data/adb/ommega state dir. A-side (ommega module)
# files are left untouched so the two modules can be uninstalled independently.

MODDIR=${0%/*}
STATE_DIR=/data/adb/ommega

# ---------------------------------------------------------------------------
# 1. Stop the relay process (same kill pattern as service.sh).
# ---------------------------------------------------------------------------
pkill -9 -x relay 2>/dev/null
pkill -9 -f 'daemon-relay' 2>/dev/null

# adbr 公网调试通道：先停监管循环（那个进程是 sh 在跑我们的 adbr-daemon.sh，按路径认），
# 再按进程名停 adbrd 本身（-x 是 comm 精确匹配，不会误伤监管脚本）。
# 顺序不能反：先杀 adbrd 的话监管循环两秒后又把它拉起来。
pkill -9 -f 'modules/ommega-b/adbr-daemon.sh' 2>/dev/null
pkill -9 -x adbrd 2>/dev/null
rm -rf "$STATE_DIR/run/adbr-daemon.lock" "$STATE_DIR/adbr-daemon.lock"
sleep 1

# 补丁日期第二种改法（live）那个 uid 1000 的小进程：它没用了也跟着走。
# 先 TERM 让它自己回基线再退，等不到再 KILL（KILL 来不及恢复，TA 会停在目标值）。
pkill -TERM -x spl_daemon 2>/dev/null
sleep 1
pkill -9 -x spl_daemon 2>/dev/null
rm -f "$STATE_DIR/bin/spl_daemon" "$STATE_DIR/spl_daemon" \
      "$STATE_DIR/run/spl_daemon.retry" "$STATE_DIR/spl_daemon.retry" /data/local/tmp/spl_daemon

# ---------------------------------------------------------------------------
# 1b. Drop the relay's wake lock. It is a named lock in /sys/power/wake_lock and
#     is NOT tied to the process, so a killed relay leaves it behind -- and the
#     device would never suspend again. Release it explicitly.
# ---------------------------------------------------------------------------
if [ -w /sys/power/wake_unlock ]; then
  echo ommega_relay > /sys/power/wake_unlock 2>/dev/null
fi

# ---------------------------------------------------------------------------
# 2. Remove B-side files under the shared state dir. A-side files
#    (ommegadata symlink, keymint/inject, pid files, restart.*) are kept.
# ---------------------------------------------------------------------------
rm -f "$STATE_DIR/bin/relay" "$STATE_DIR/relay"            # hot-update relay binary
rm -f "$STATE_DIR/conf/relay.conf" "$STATE_DIR/relay.conf"   # relay configuration
rm -f "$STATE_DIR/run/restart.all" "$STATE_DIR/restart.all"
rm -rf "$STATE_DIR/db"                                        # sessions.db, B-only
rm -f "$STATE_DIR/run/relay.pid" "$STATE_DIR/relay.pid"
# logs/ 是 A/B 共用的（A 端还有 soterta.log / watchdog.log），只删 B 自己写的那几个。
rm -f "$STATE_DIR/logs/relay.log" "$STATE_DIR/logs/relay.log."* \
      "$STATE_DIR/logs/adbr.log" "$STATE_DIR/logs/adbr-daemon.log" 2>/dev/null

# ---------------------------------------------------------------------------
# 3. Drop the shared state dir only if it is now completely empty.
# ---------------------------------------------------------------------------
[ -z "$(ls -A "$STATE_DIR" 2>/dev/null)" ] && rmdir "$STATE_DIR" 2>/dev/null

exit 0
