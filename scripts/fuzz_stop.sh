#!/usr/bin/env bash
set -euo pipefail

PID_FILE="/tmp/fuzz-continuous-master.pid"

if [ -f "$PID_FILE" ]; then
  PID="$(cat "$PID_FILE")"
  if [ -n "$PID" ]; then
    kill "$PID" 2>/dev/null || true
  fi
  rm -f "$PID_FILE"
fi

pkill -f 'bash ./scripts/run_continuous_fuzz.sh' 2>/dev/null || true
pkill -f 'cargo \+nightly fuzz run fuzz_target_' 2>/dev/null || true

echo "stopped"
