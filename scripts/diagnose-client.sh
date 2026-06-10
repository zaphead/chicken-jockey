#!/usr/bin/env bash
# Runs headless pipeline checks and writes a readable report (no visual inspection).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
REPORT="${CJ_DIAGNOSE_REPORT:-/tmp/chicken-jockey-diagnose.txt}"
LOG="${CJ_CLIENT_LOG:-/tmp/chicken-jockey-client-run.log}"

cd "$ROOT"
source "$HOME/.cargo/env" 2>/dev/null || true

{
  echo "=== Chicken Jockey client diagnose ==="
  echo "time: $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
  echo

  echo "--- headless binary (client-diagnose) ---"
  if RUST_LOG=info cargo run -q --bin client-diagnose 2>&1; then
    echo "headless: PASS"
  else
    echo "headless: FAIL"
  fi
  echo

  echo "--- integration test (headless_pipeline) ---"
  if cargo test -p client headless_pipeline -- --nocapture 2>&1; then
    echo "integration_test: PASS"
  else
    echo "integration_test: FAIL"
  fi
  echo

  echo "--- live client sample (15s, CJ_DIAGNOSTIC=1) ---"
  cargo build -p client -q
  : >"$LOG"
  RUST_LOG=info CJ_DIAGNOSTIC=1 cargo run -p client --bin client >>"$LOG" 2>&1 &
  PID=$!
  sleep 15
  kill "$PID" 2>/dev/null || true
  wait "$PID" 2>/dev/null || true
  echo "client log: $LOG"
  grep "cj diag:" "$LOG" | tail -20 || echo "(no diagnostic lines)"
  if grep -q "vertices=[1-9]" "$LOG" || grep -q "vertices=[0-9][0-9]" "$LOG"; then
    echo "live_sample: meshes reported in log"
  else
    echo "live_sample: WARNING — no non-zero vertex count in diagnostic lines"
  fi
} | tee "$REPORT"

echo
echo "Report written to: $REPORT"
