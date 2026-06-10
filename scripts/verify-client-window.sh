#!/usr/bin/env bash
# Verifies the native client opens a visible window on macOS.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
LOG="/tmp/verify-chicken-jockey-client.log"
VERDICT="/tmp/verify-chicken-jockey-verdict.txt"
TIMEOUT_SEC=15

cd "$ROOT"
source "$HOME/.cargo/env" 2>/dev/null || true

cargo build -p client -q

: >"$LOG"
RUST_LOG=info cargo run -p client --bin client >>"$LOG" 2>&1 &
PID=$!

cleanup() {
  kill "$PID" 2>/dev/null || true
  wait "$PID" 2>/dev/null || true
}
trap cleanup EXIT

deadline=$((SECONDS + TIMEOUT_SEC))
renderer_ready=0
while (( SECONDS < deadline )); do
  if grep -q "renderer ready" "$LOG"; then
    renderer_ready=1
    break
  fi
  if ! kill -0 "$PID" 2>/dev/null; then
    break
  fi
  sleep 0.25
done

sleep 2

probe="$(swift -e '
import CoreGraphics
import Foundation
var best = (title: "", width: 0, height: 0, onScreen: false)
let lists: [(CGWindowListOption, Bool)] = [(.optionOnScreenOnly, true), (.optionAll, false)]
for (option, onScreen) in lists {
  let info = CGWindowListCopyWindowInfo(option, kCGNullWindowID) as? [[String: Any]] ?? []
  for w in info {
    guard let owner = w[kCGWindowOwnerName as String] as? String, owner == "client" else { continue }
    let name = w[kCGWindowName as String] as? String ?? ""
    let bounds = w[kCGWindowBounds as String] as? [String: CGFloat] ?? [:]
    let width = Int(bounds["Width"] ?? 0)
    let height = Int(bounds["Height"] ?? 0)
    let area = width * height
    let bestArea = best.width * best.height
    if area > bestArea { best = (name, width, height, onScreen) }
  }
}
print("\(best.title)|\(best.width)|\(best.height)|\(best.onScreen)")
' 2>/dev/null || echo "|0|0|false")"

IFS='|' read -r window_title window_width window_height on_screen <<<"$probe"

{
  echo "claim=Client shows visible Chicken Jockey window within ${TIMEOUT_SEC}s"
  echo "pid=$PID"
  echo "renderer_ready=$renderer_ready"
  echo "window_title=$window_title"
  echo "width=$window_width"
  echo "height=$window_height"
  echo "on_screen=$on_screen"
  echo "--- client log ---"
  cat "$LOG"
} >"$VERDICT"

if (( renderer_ready == 1 )) && (( window_width > 100 && window_height > 100 )) && [[ "$on_screen" == "true" ]]; then
  echo "VERIFIED"
  exit 0
fi

echo "NOT VERIFIED"
exit 1
