#!/system/bin/sh
# B 端运行时目录整理：把平铺在 /data/adb/ommega 根下的东西收进 bin/conf/run/db/logs。
# 只支持新路径；幂等，旧文件有、新位置没东西就搬过去。

STATE_DIR=/data/adb/ommega
BIN_DIR=$STATE_DIR/bin
CONF_DIR=$STATE_DIR/conf
RUN_DIR=$STATE_DIR/run
DB_DIR=$STATE_DIR/db

migrate_ommega_b_layout() {
  mkdir -p "$BIN_DIR" "$CONF_DIR" "$RUN_DIR" "$DB_DIR" "$STATE_DIR/logs" 2>/dev/null

  for f in relay resetprop adbrd spl_daemon; do do
    [ -f "$STATE_DIR/$f" ] || continue
    [ -e "$BIN_DIR/$f" ] || mv -f "$STATE_DIR/$f" "$BIN_DIR/$f" 2>/dev/null
  done

  for f in relay.conf; do
    [ -f "$STATE_DIR/$f" ] || continue
    [ -e "$CONF_DIR/$f" ] || mv -f "$STATE_DIR/$f" "$CONF_DIR/$f" 2>/dev/null
  done

  for f in relay.pid restart.all restart.keymint restart.injector spl_daemon.retry; do do
    [ -f "$STATE_DIR/$f" ] || continue
    [ -e "$RUN_DIR/$f" ] || mv -f "$STATE_DIR/$f" "$RUN_DIR/$f" 2>/dev/null
  done
  for d in relay-service.lock adbr-daemon.lock; do
    [ -d "$STATE_DIR/$d" ] || continue
    [ -e "$RUN_DIR/$d" ] || mv -f "$STATE_DIR/$d" "$RUN_DIR/$d" 2>/dev/null
  done

  for f in sessions.db sessions.db-wal sessions.db-shm; do
    [ -f "$STATE_DIR/$f" ] || continue
    [ -e "$DB_DIR/$f" ] || mv -f "$STATE_DIR/$f" "$DB_DIR/$f" 2>/dev/null
  done
  [ -d "$STATE_DIR/sessions" ] && [ ! -e "$DB_DIR/sessions" ] && \
    mv -f "$STATE_DIR/sessions" "$DB_DIR/sessions" 2>/dev/null
}

migrate_ommega_b_layout
