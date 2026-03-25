#!/bin/bash
# Run OpenProof against miniF2F-test problems (244 problems).
# Usage: ./benchmarks/run_minif2f.sh [timeout_secs] [max_problems] [start_from]

set -euo pipefail
cd "$(dirname "$0")/.."

TIMEOUT=${1:-300}
MAX=${2:-0}
START=${3:-0}
RESULTS_DIR="benchmarks/results/$(date +%Y%m%d_%H%M%S)"
mkdir -p "$RESULTS_DIR"

MINIF2F="benchmarks/miniF2F-lean4/MiniF2F/Test"
if [ ! -d "$MINIF2F" ]; then
    echo "miniF2F not found. Run: git clone https://github.com/yangky11/miniF2F-lean4.git benchmarks/miniF2F-lean4"
    exit 1
fi

PROBLEMS=$(find "$MINIF2F" -name "*.lean" | sort)
TOTAL=$(echo "$PROBLEMS" | wc -l | tr -d ' ')
echo "Found $TOTAL test problems (timeout=${TIMEOUT}s per problem)"

if [ "$START" -gt 0 ]; then
    PROBLEMS=$(echo "$PROBLEMS" | tail -n +$((START + 1)))
fi
if [ "$MAX" -gt 0 ]; then
    PROBLEMS=$(echo "$PROBLEMS" | head -n "$MAX")
    TOTAL=$MAX
fi

SOLVED=0
FAILED=0
ERRORED=0
IDX=0

for PROBLEM_FILE in $PROBLEMS; do
    IDX=$((IDX + 1))
    PROBLEM_NAME=$(basename "$PROBLEM_FILE" .lean)

    # Extract theorem statement (everything between 'theorem' and ':= by sorry')
    STATEMENT=$(sed -n '/^theorem\|^lemma/,/:= by sorry/p' "$PROBLEM_FILE" | tr '\n' ' ' | sed 's/:= by sorry.*//')
    if [ -z "$STATEMENT" ]; then
        echo "[$IDX/$TOTAL] $PROBLEM_NAME ... SKIP (no theorem)"
        continue
    fi

    echo -n "[$IDX/$TOTAL] $PROBLEM_NAME ... "

    START_TIME=$(date +%s)
    LOG="$RESULTS_DIR/${PROBLEM_NAME}.log"

    timeout "$TIMEOUT" cargo run -q -- run --problem "Prove in Lean 4: $STATEMENT" > "$LOG" 2>&1 || true
    END_TIME=$(date +%s)
    ELAPSED=$((END_TIME - START_TIME))

    if grep -q "All proof nodes verified\|All nodes verified\|DIRECT VERIFICATION SUCCEEDED" "$LOG" 2>/dev/null; then
        echo "SOLVED (${ELAPSED}s)"
        SOLVED=$((SOLVED + 1))
        echo "SOLVED $ELAPSED $PROBLEM_NAME" >> "$RESULTS_DIR/summary.txt"
    elif [ "$ELAPSED" -ge "$TIMEOUT" ]; then
        echo "TIMEOUT (${ELAPSED}s)"
        ERRORED=$((ERRORED + 1))
        echo "TIMEOUT $ELAPSED $PROBLEM_NAME" >> "$RESULTS_DIR/summary.txt"
    else
        echo "FAILED (${ELAPSED}s)"
        FAILED=$((FAILED + 1))
        echo "FAILED $ELAPSED $PROBLEM_NAME" >> "$RESULTS_DIR/summary.txt"
    fi

    ATTEMPTED=$((SOLVED + FAILED + ERRORED))
    if [ "$ATTEMPTED" -gt 0 ]; then
        PCT=$((SOLVED * 100 / ATTEMPTED))
        echo "  Running: $SOLVED/$ATTEMPTED ($PCT%) solved"
    fi
done

echo ""
echo "==============================="
echo "  miniF2F Benchmark Results"
echo "==============================="
echo "Solved:  $SOLVED / $((SOLVED + FAILED + ERRORED))"
echo "Failed:  $FAILED"
echo "Timeout: $ERRORED"
if [ "$((SOLVED + FAILED + ERRORED))" -gt 0 ]; then
    echo "Rate:    $((SOLVED * 100 / (SOLVED + FAILED + ERRORED)))%"
fi
echo "Results: $RESULTS_DIR/"
echo "==============================="
