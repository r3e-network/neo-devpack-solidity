#!/usr/bin/env bash
set -euo pipefail

PID_FILE="/tmp/fuzz-continuous-master.pid"
LOG_FILE="/tmp/fuzz-continuous-master.log"

if [ ! -f "$PID_FILE" ]; then
  echo "status=stopped"
  echo "reason=pid_file_missing"
  exit 0
fi

PID="$(cat "$PID_FILE")"
if [ -z "$PID" ] || ! kill -0 "$PID" 2>/dev/null; then
  echo "status=stopped"
  echo "pid=${PID:-unknown}"
  echo "reason=process_missing"
  exit 0
fi

echo "status=running"
echo "pid=$PID"
ps -fp "$PID" || true

echo
echo "child_processes:"
ps -ef | rg 'run_continuous_fuzz|cargo \+nightly fuzz run fuzz_target_' || true

if [ -f "$LOG_FILE" ]; then
  echo
  echo "last_log_lines:"
  tail -n 20 "$LOG_FILE"
fi
