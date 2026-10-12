#!/system/bin/sh

# adbrd 监管脚本 —— A / B 两端共用同一份，用参数选读哪份配置：
#
#   sh adbr-daemon.sh --a   A 端：/data/misc/keystore/ommega/config 里的 `adbr:`
#   sh adbr-daemon.sh --b   B 端：/data/adb/ommega/relay.conf 里的 OMMEGA_RELAY_ABDR_ENABLED
#
# 设备被公网远控这件事必须由用户显式点头，所以这里只认配置里的开关，不开就不跑：
#   开关开着 → 保证有一个 adbr 在跑（掉了 2 秒后重拉）
#   开关关着 → 把在跑的那个停掉
# 开关是每 2 秒重读一次，所以 WebUI 里一改，最多 2 秒就生效，不用重启设备。
#
# 认进程认就一个名字 `adbrd` —— 设备侧那份就叫这个（`adbr` 是电脑上的命令行客户端，
# 不是设备上的东西）。认错了名字开关就管不住：名字对不上时，关掉停不掉、打开又会
# 在旁边再起一条，同一台设备在服务端就成了两条通道。
#
# 二进制只有 arm64 一份（见仓库 vendor/adbrd/README.md），别的架构直接退出。

MODE=""
case "$1" in
  --a|a) MODE=a ;;
  --b|b) MODE=b ;;
  *) echo "[adbr] 用法: sh adbr-daemon.sh --a|--b" >&2; exit 2 ;;
esac

MODDIR=${0%/*}
STATE_DIR=/data/adb/ommega
LOG="$STATE_DIR/logs/adbr.log"
LOCK_DIR="$STATE_DIR/adbr-daemon.lock"

if [ "$MODE" = "a" ]; then
  CONF=/data/misc/keystore/ommega/config
else
  CONF="$STATE_DIR/relay.conf"
fi

mkdir -p "$STATE_DIR/logs"

# 只认 arm64：上游只给了这一份构建。
abi=$(getprop ro.product.cpu.abi 2>/dev/null)
case "$abi" in
  arm64*|aarch64*) : ;;
  *)
    echo "[adbr] 本机不是 arm64（$abi），adbr 不启用" >&2
    exit 0
    ;;
esac

# 单例：service.sh 每次跑都会把我们拉一遍，只有一份能持有这个锁。
# 陈锁（持有者没了：强杀、断电）清掉重抢。
if mkdir "$LOCK_DIR" 2>/dev/null; then
  echo "$$" > "$LOCK_DIR/pid"
else
  holder=$(cat "$LOCK_DIR/pid" 2>/dev/null)
  if [ -n "$holder" ] && kill -0 "$holder" 2>/dev/null; then
    exit 0
  fi
  rm -rf "$LOCK_DIR" 2>/dev/null
  mkdir "$LOCK_DIR" 2>/dev/null || exit 0
  echo "$$" > "$LOCK_DIR/pid"
fi
trap 'rm -rf "$LOCK_DIR"' EXIT INT TERM HUP

truthy() {
  case "$(printf '%s' "$1" | tr -d ' \t\r' | tr 'A-Z' 'a-z')" in
    1|true|yes|on) return 0 ;;
  esac
  return 1
}

# 配置里的开关。没写这个键 = 关（adbr 是额外开的口子，绝不能不写就自己开）。
switch_on() {
  if [ "$MODE" = "a" ]; then
    value=$(sed -n 's/^[[:space:]]*adbr[[:space:]]*:[[:space:]]*//p' "$CONF" 2>/dev/null | head -n 1)
  else
    value=$(sed -n 's/^OMMEGA_RELAY_ABDR_ENABLED=//p' "$CONF" 2>/dev/null | tail -n 1)
  fi
  truthy "$value"
}

# 用 /proc/<pid>/comm 认进程，不用 pidof/pgrep —— 那两个在 toybox 上行为不一致，
# 而且 pkill -f 会把带 "adbr" 的脚本（就是本脚本自己）一起杀掉。
# 只认 `adbrd`：设备侧那份就叫这个（`adbr` 是电脑上的客户端）。
adbr_pids() {
  for d in /proc/[0-9]*; do
    [ -r "$d/comm" ] || continue
    if [ "$(cat "$d/comm" 2>/dev/null)" = "adbrd" ]; then
      printf '%s\n' "${d#/proc/}"
    fi
  done
}

find_bin() {
  if [ -x "$STATE_DIR/adbrd" ]; then
    printf '%s\n' "$STATE_DIR/adbrd"
    return 0
  fi
  for abi in arm64-v8a; do
    if [ -f "$MODDIR/libs/$abi/adbrd" ]; then
      printf '%s\n' "$MODDIR/libs/$abi/adbrd"
      return 0
    fi
  done
  return 1
}

while true; do
  if switch_on; then
    if [ -z "$(adbr_pids)" ]; then
      bin=$(find_bin)
      if [ -n "$bin" ]; then
        chmod 0755 "$bin" 2>/dev/null || true
        # --foreground：别让它自己 detach，输出直接进我们的日志，监管循环也看得见它。
        "$bin" --foreground >> "$LOG" 2>&1 &
        echo "[adbr] 已拉起 $bin (pid $!)"
      else
        echo "[adbr] 开关开着但找不到 adbr 二进制"
      fi
    fi
  else
    pids=$(adbr_pids)
    if [ -n "$pids" ]; then
      echo "[adbr] 开关关着，停掉: $pids"
      for p in $pids; do
        exe=$(readlink "/proc/$p/exe" 2>/dev/null)
        case "$exe" in
          "$STATE_DIR"/*|"$MODDIR"/*) : ;;
          *) echo "[adbr] 停掉的是 $exe（不是模块目录里的那份，确认一下是不是别处手装的）" ;;
        esac
      done
      # shellcheck disable=SC2086
      kill $pids 2>/dev/null
    fi
  fi
  sleep 2
done
