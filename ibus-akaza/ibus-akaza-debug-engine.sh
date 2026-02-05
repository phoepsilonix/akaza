#!/bin/bash
set -ex

BASEDIR=$(dirname "$0")

umask 077

LOG_PATH="${HOME}/.cache/akaza/logs/ibus-akaza.log"
exec 1>> "$LOG_PATH"
exec 2>&1

export RUST_BACKTRACE=4

MEM_LIMIT_MB="${AKAZA_DEBUG_MEM_LIMIT_MB:-2048}"
if ! ulimit -v "$((MEM_LIMIT_MB * 1024))"; then
  echo "warning: failed to set ulimit -v (MEM_LIMIT_MB=${MEM_LIMIT_MB})" >&2
fi

exec $BASEDIR/../target/release/ibus-akaza --ibus -vv
