MODDIR=${0%/*}
STATE_DIR=/data/adb/ommega
PROP_FILE="$MODDIR/module.prop"

mkdir -p "$STATE_DIR" "$STATE_DIR/logs"

# Update the module.prop description so the module card reflects runtime state.
# Mirrors the legacy A-side (client-a) behaviour.
update_status() {
  local status="$1"
  [ -f "$PROP_FILE" ] || return 0
  sed -i 's/^description=.*/description='"$status"'/' "$PROP_FILE" 2>/dev/null || true
}

pid_matches_script() {
  pid=$1
  script=$2
  [ -r "/proc/$pid/cmdline" ] || return 1
  cmdline=$(tr '\0' ' ' < "/proc/$pid/cmdline" 2>/dev/null)
  echo "$cmdline" | grep -F "$script" >/dev/null 2>&1
}

start_daemon() {
  script=$1
  pidfile=$2

  if [ -f "$pidfile" ]; then
    pid=$(cat "$pidfile" 2>/dev/null)
    if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null && pid_matches_script "$pid" "$script"; then
      return 0
    fi
    rm -f "$pidfile"
  fi

  sh "$script" &
  pid=$!
  echo $pid > "$pidfile"
  sleep 1
  if ! kill -0 "$pid" 2>/dev/null || ! pid_matches_script "$pid" "$script"; then
    rm -f "$pidfile"
    return 1
  fi
  return 0
}

# WebUI is served by the KernelSU/APatch manager, which auto-detects the
# module's webroot/ folder and injects the window.ksu bridge. No separate
# web server is needed.

# Unconditionally clear any leftover module processes so a fresh start never
# coexists with a stale instance (e.g. after a module update or a manual
# restart where the old watchdog died but its children survived). Only
# module-owned paths are matched; the system keymint HAL is never touched.
kill_all() {
  pkill -9 -f 'modules/ommega/daemon' 2>/dev/null
  pkill -9 -f 'modules/ommega/libs' 2>/dev/null
  pkill -9 -f "$STATE_DIR/keymint" 2>/dev/null
  pkill -9 -f "$STATE_DIR/ommega-inject" 2>/dev/null
  sleep 1
}

# post-fs-data 阶段可能已经把两个守护拉起来了（为了抢在 keystore2 之前守着，好接住开机
# 解锁那份材料）。那两个还活着就别 kill 重来 —— 那会把刚抢到的时间窗白白丢掉。
daemons_alive() {
  for spec in "$STATE_DIR/keymint-daemon.pid:$MODDIR/daemon" "$STATE_DIR/injector-daemon.pid:$MODDIR/daemon-injector"; do
    pf=${spec%%:*}
    sp=${spec#*:}
    [ -f "$pf" ] || return 1
    p=$(cat "$pf" 2>/dev/null)
    [ -n "$p" ] && kill -0 "$p" 2>/dev/null && pid_matches_script "$p" "$sp" || return 1
  done
  return 0
}

if daemons_alive; then
  update_status "Ommega ✅ 运行中"
else
  kill_all
  update_status "Ommega ⏳ 启动中"
  start_daemon "$MODDIR/daemon" "$STATE_DIR/keymint-daemon.pid"
  start_daemon "$MODDIR/daemon-injector" "$STATE_DIR/injector-daemon.pid"
fi

# Bundled PathMask kernel module: pick the .ko matching this kernel and hide
# /system/priv-app/SoterService only when its binder service cannot be reached
# (see kmod-loader.sh).  Detached on purpose: it has to finish well within a few
# seconds and must never delay late_start or the daemons above.
if [ -f "$MODDIR/kmod-loader.sh" ]; then
  # 调试日志关着时连这个日志文件都不建（它也是「日志」，开关开了才有）。
  kmod_log=/dev/null
  kmod_val=$(grep -iE '^[[:space:]]*(debug_logging|debug|verbose)[[:space:]]*:' /data/misc/keystore/ommega/config 2>/dev/null | head -n1 | sed 's/^[^:]*:[[:space:]]*//' | tr -d '\r' | tr 'A-Z' 'a-z')
  case "$kmod_val" in
    1|true|yes|on) kmod_log="$STATE_DIR/kmod-loader.log" ;;
  esac
  ( sh "$MODDIR/kmod-loader.sh" >> "$kmod_log" 2>&1 & )
fi

# adbr 公网调试通道（默认关，WebUI 里点头才跑）：单独一个监管循环，跟 keymint/
# injector 那两根线互不影响；它自己拿锁，service.sh 被拉多次也不会长出第二份。
if [ -f "$MODDIR/adbr-daemon.sh" ]; then
  ( sh "$MODDIR/adbr-daemon.sh" --a >> "$STATE_DIR/logs/adbr-daemon.log" 2>&1 & )
fi

# 上面两个守护脚本各自会看住自己的子进程，但**它们自己被系统清掉**（低内存 / 清理类 App）
# 之后就没东西再补了 —— 症状是手机看着一切正常，而 keymint/injector 或 adbr 那层
# 静默地永久消失。这里常驻一个守护循环，每 5 秒看一次，谁不在补谁。
daemon_one_alive() {
  script=$1
  pidfile=$2
  [ -f "$pidfile" ] || return 1
  p=$(cat "$pidfile" 2>/dev/null)
  [ -n "$p" ] && kill -0 "$p" 2>/dev/null && pid_matches_script "$p" "$script"
}

keep_daemons_alive() {
  fixed=0
  for spec in "$MODDIR/daemon:$STATE_DIR/keymint-daemon.pid" "$MODDIR/daemon-injector:$STATE_DIR/injector-daemon.pid"; do
    sp=${spec%%:*}
    pf=${spec#*:}
    if ! daemon_one_alive "$sp" "$pf"; then
      start_daemon "$sp" "$pf" && fixed=1
    fi
  done
  if [ -f "$MODDIR/adbr-daemon.sh" ]; then
    holder=$(cat "$STATE_DIR/adbr-daemon.lock/pid" 2>/dev/null)
    if [ -z "$holder" ] || ! kill -0 "$holder" 2>/dev/null; then
      rm -rf "$STATE_DIR/adbr-daemon.lock" 2>/dev/null
      ( sh "$MODDIR/adbr-daemon.sh" --a >> "$STATE_DIR/logs/adbr-daemon.log" 2>&1 & )
      fixed=1
    fi
  fi
  [ "$fixed" = 1 ] && update_status "Ommega ✅ 运行中"
  return 0
}

(
  while true; do
    keep_daemons_alive
    sleep 5
  done
) >> "$STATE_DIR/logs/watchdog.log" 2>&1 &
