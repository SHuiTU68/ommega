#!/system/bin/sh
# 软件 RKP HAL 的开关与守护。
#
# 原厂 `android.hardware.security.keymint.IRemotelyProvisionedComponent/default` 只报
# versionNumber=1（IRPC V1，没有 generateCertificateRequestV2），工程模式的 Key 状态页
# 按 V3 调，每次都回 Status(-8)。这里注册同名服务把它顶掉：getHardwareInfo 报 V3，
# generateCertificateRequestV2 转给 daemon 出 CSR。
#
# 跟 SOTER 的接管不一样：原厂 RKP 就在 keymint 进程里，不能停（停了 keystore 全挂），
# 所以这里只做「注册同名服务」+ 保活，servicemanager 后注册的覆盖先注册的。
#
#   rkpta.sh             converge 一次（service.sh 每轮调）
#   rkpta.sh status      打一行状态
#
# 开关是 config 里的 `rkp_inject`（1/0，默认开）；关掉就杀守护、把服务名让回原厂。

MODDIR=${0%/*}
STATE_DIR=/data/adb/ommega
CONF=${OMMEGA_CONF_PATH:-/data/misc/keystore/ommega/config}
PIDFILE=$STATE_DIR/rkpta.pid
LOG=$STATE_DIR/logs/rkpta.log
SERVICE=android.hardware.security.keymint.IRemotelyProvisionedComponent/default
BIN=

log() {
  mkdir -p "$STATE_DIR/logs" 2>/dev/null
  printf '%s %s\n' "$(date -u '+%Y-%m-%d %H:%M:%S')" "$*" >> "$LOG" 2>/dev/null
}

conf_get() {
  [ -f "$CONF" ] || return 0
  sed -n "s/^[[:space:]]*$1[[:space:]]*:[[:space:]]*\(.*\)$/\1/p" "$CONF" 2>/dev/null | head -n1
}

enabled() {
  case "$(conf_get rkp_inject | tr 'A-Z' 'a-z' | tr -d ' \r')" in
    0|false|no|off) return 1 ;;
  esac
  return 0
}

device_abi() {
  abi=$(getprop ro.product.cpu.abi 2>/dev/null)
  case "$abi" in
    arm64*|aarch64*) echo arm64-v8a ;;
    armeabi*|armv7*) echo armeabi-v7a ;;
    x86_64*) echo x86_64 ;;
    i?86|x86) echo x86 ;;
    *) echo "" ;;
  esac
}

find_bin() {
  [ -n "$BIN" ] && return 0
  for candidate in "$STATE_DIR/ommega-inject" "$MODDIR/ommega-inject"; do
    if [ -x "$candidate" ]; then BIN=$candidate; return 0; fi
  done
  abi=$(device_abi)
  if [ -n "$abi" ] && [ -x "$MODDIR/libs/$abi/ommega-inject" ]; then
    BIN=$MODDIR/libs/$abi/ommega-inject
    return 0
  fi
  return 1
}

daemon_pid() {
  [ -f "$PIDFILE" ] || return 1
  pid=$(cat "$PIDFILE" 2>/dev/null)
  case "$pid" in
    ''|*[!0-9]*) return 1 ;;
  esac
  kill -0 "$pid" 2>/dev/null || return 1
  tr '\0' ' ' < "/proc/$pid/cmdline" 2>/dev/null | grep -q 'rkpta-serve' || return 1
  echo "$pid"
}

stop_daemon() {
  pid=$(daemon_pid)
  if [ -n "$pid" ]; then
    kill -TERM "$pid" 2>/dev/null
    i=0
    while [ "$i" -lt 10 ] && kill -0 "$pid" 2>/dev/null; do
      sleep 0.3
      i=$((i + 1))
    done
    kill -0 "$pid" 2>/dev/null && kill -KILL "$pid" 2>/dev/null
  fi
  rm -f "$PIDFILE" 2>/dev/null
}

start_daemon() {
  find_bin || { log "injector binary not found"; return 1; }
  log "starting software RKP HAL: $BIN"
  "$BIN" --rkpta-serve >>"$LOG" 2>&1 &
  pid=$!
  echo "$pid" > "$PIDFILE.tmp" 2>/dev/null
  mv -f "$PIDFILE.tmp" "$PIDFILE" 2>/dev/null
  sleep 1
  daemon_pid >/dev/null
}

case "$1" in
  status)
    if [ -n "$(daemon_pid)" ]; then
      echo "rkpta=us service=$(service check "$SERVICE" 2>/dev/null | sed 's/.*: //')"
    else
      echo "rkpta=none service=$(service check "$SERVICE" 2>/dev/null | sed 's/.*: //')"
    fi
    exit 0
    ;;
  enable)
    enabled || exit 0
    start_daemon
    exit $?
    ;;
  disable)
    stop_daemon
    exit 0
    ;;
esac

# 默认：一次 converge（service.sh 的 keep-alive 循环会反复调）。
if enabled; then
  if [ -z "$(daemon_pid)" ]; then
    start_daemon
  fi
else
  stop_daemon
fi
