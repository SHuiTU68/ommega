#!/system/bin/sh
# 运行时目录整理：把原来平铺在 /data/adb/ommega 根下的东西收进 bin/conf/run/db/logs。
#
# 只在开机（post-fs-data / service）和安装时各跑一次；幂等：目标已存在就不动，
# 旧路径有、新路径没有就搬过去。**只支持新路径**，不再保留老的读取兜底。

STATE_DIR=/data/adb/ommega
BIN_DIR=$STATE_DIR/bin
CONF_DIR=$STATE_DIR/conf
RUN_DIR=$STATE_DIR/run
DB_DIR=$STATE_DIR/db
LOG_DIR_TOP=$STATE_DIR/logs

migrate_ommega_layout() {
  mkdir -p "$BIN_DIR" "$CONF_DIR" "$RUN_DIR" "$DB_DIR" "$LOG_DIR_TOP" 2>/dev/null

  # 二进制（hot update 会往这里换，模块目录里的 libs/ 仍是出厂副本）
  for f in keymint ommega-inject soterta-ta resetprop adbrd; do
    [ -f "$STATE_DIR/$f" ] || continue
    [ -e "$BIN_DIR/$f" ] || mv -f "$STATE_DIR/$f" "$BIN_DIR/$f" 2>/dev/null
  done

  # 配置 / 状态文件
  for f in props.conf injector.payload pathmask.state pathmask.owned \
           remote_identity.applied remote_identity.updated empty_permissions.xml \
           relay.conf log_flag kmod-loader.log; do
    [ -f "$STATE_DIR/$f" ] || continue
    tgt=$CONF_DIR/$f
    case "$f" in
      kmod-loader.log) tgt=$LOG_DIR_TOP/$f ;;
    esac
    [ -e "$tgt" ] || mv -f "$STATE_DIR/$f" "$tgt" 2>/dev/null
  done

  # 运行期：pid、重启标记、锁目录
  for f in keymint-daemon.pid injector-daemon.pid soterta.pid soterta.takeover.pid \
           soterta.host.kicked relay.pid restart.keymint restart.injector restart.all; do
    [ -f "$STATE_DIR/$f" ] || continue
    [ -e "$RUN_DIR/$f" ] || mv -f "$STATE_DIR/$f" "$RUN_DIR/$f" 2>/dev/null
  done
  for d in adbr-daemon.lock injector-daemon.lock keymint-daemon.lock relay-service.lock; do
    [ -d "$STATE_DIR/$d" ] || continue
    [ -e "$RUN_DIR/$d" ] || mv -f "$STATE_DIR/$d" "$RUN_DIR/$d" 2>/dev/null
  done

  # 数据库（B 端会话）+ 老的目录式会话库
  for f in sessions.db sessions.db-wal sessions.db-shm; do
    [ -f "$STATE_DIR/$f" ] || continue
    [ -e "$DB_DIR/$f" ] || mv -f "$STATE_DIR/$f" "$DB_DIR/$f" 2>/dev/null
  done
  [ -d "$STATE_DIR/sessions" ] && [ ! -e "$DB_DIR/sessions" ] && \
    mv -f "$STATE_DIR/sessions" "$DB_DIR/sessions" 2>/dev/null

  # keystore 侧：备份类收进 backups/，配置与密钥材料留在原地（TA 里路径是写死的）
  KS_DIR=/data/misc/keystore/ommega
  if [ -d "$KS_DIR" ]; then
    mkdir -p "$KS_DIR/backups" 2>/dev/null
    for f in "$KS_DIR"/config.bak* "$KS_DIR"/config.toml.bak "$KS_DIR"/keybox.xml.bak; do
      [ -f "$f" ] || continue
      mv -f "$f" "$KS_DIR/backups/" 2>/dev/null
    done
  fi
}

migrate_ommega_layout
