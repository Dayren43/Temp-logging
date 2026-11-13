#!/usr/bin/env bash
#
# Gated deploy for the ESP32-C3 sensor firmware.
#
#   1. Build + flash the self-test, watch the serial verdict.
#   2. If it PASSES  -> build + flash the real firmware, save it as last-known-good.
#      If it FAILS   -> re-flash the saved last-known-good firmware (rollback).
#
# Credentials come from secrets.env (gitignored) or the environment.
# Run from anywhere:  bash deploy.sh        (needs git-bash on Windows)

set -euo pipefail
cd "$(dirname "$0")"

# ---- credentials ----
if [ -f secrets.env ]; then
    # shellcheck disable=SC1091
    . ./secrets.env
fi
: "${SSID:?Set SSID in secrets.env or the environment}"
: "${PASSWORD:?Set PASSWORD in secrets.env or the environment}"
export SSID PASSWORD

# ---- paths ----
TARGET="riscv32imc-unknown-none-elf"
RELDIR="target/${TARGET}/release"
SELFTEST_BIN="${RELDIR}/selftest"
FIRMWARE_BIN="${RELDIR}/esp32c3-embassy"
LASTGOOD="deploy/lastgood.elf"
SELFTEST_TIMEOUT=120   # max seconds to wait for the PASS/FAIL verdict
mkdir -p deploy

# ---- 1. build + run the self-test ----
echo "==> Building self-test..."
cargo build --release --bin selftest

echo "==> Flashing self-test and watching for the verdict (up to ${SELFTEST_TIMEOUT}s)..."
LOG="$(mktemp)"
espflash flash --monitor "$SELFTEST_BIN" >"$LOG" 2>&1 &
MON_PID=$!

RESULT="timeout"
for _ in $(seq 1 "$SELFTEST_TIMEOUT"); do
    if grep -q "ALL TESTS PASSED" "$LOG"; then RESULT="pass"; break; fi
    if grep -q "SOME TESTS FAILED" "$LOG"; then RESULT="fail"; break; fi
    if ! kill -0 "$MON_PID" 2>/dev/null; then RESULT="monitor_exited"; break; fi
    sleep 1
done

# stop the monitor and free the serial port
kill "$MON_PID" 2>/dev/null || true
wait "$MON_PID" 2>/dev/null || true
sleep 1

echo "---- self-test output ----"
grep -aE "\[[0-9]\]|RESULTS|ALL TESTS|SOME TESTS" "$LOG" || echo "(no test lines captured)"
echo "--------------------------"
rm -f "$LOG"

# ---- 2. act on the verdict ----
if [ "$RESULT" = "pass" ]; then
    echo "==> Self-test PASSED. Building and deploying firmware..."
    cargo build --release --bin esp32c3-embassy
    espflash flash "$FIRMWARE_BIN"
    cp -f "$FIRMWARE_BIN" "$LASTGOOD"
    echo "==> Deployed new firmware (saved as ${LASTGOOD})."
else
    echo "==> Self-test did not pass (result: ${RESULT})."
    if [ -f "$LASTGOOD" ]; then
        echo "==> Rolling back to last-known-good firmware..."
        espflash flash "$LASTGOOD"
        echo "==> Restored last-known-good firmware."
    else
        echo "!! No last-known-good firmware to roll back to (${LASTGOOD} missing)."
        echo "!! The board currently holds the self-test image; deploy manually once tests pass."
        exit 1
    fi
fi
