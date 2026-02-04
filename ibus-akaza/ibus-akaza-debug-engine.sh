#!/bin/bash
set -ex

BASEDIR=$(dirname "$0")

umask 077

LOG_PATH="${HOME}/.cache/akaza/logs/ibus-akaza.log"
exec 1>> "$LOG_PATH"
exec 2>&1

export RUST_BACKTRACE=4

exec $BASEDIR/../target/release/ibus-akaza --ibus -vv
