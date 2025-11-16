#!/usr/bin/env bash
set -euo pipefail

# Script location and project root
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
PROJECT_ROOT="$SCRIPT_DIR/.."
cd "$PROJECT_ROOT"

# Where to poll for mdBook being ready
SERVER_URL="http://localhost:3000/"

echo "📚 Building Rust API docs..."
cargo doc --no-deps --release

# Decide where mdBook will write its output after build
# If you have the canonical layout (book/book/) use that; otherwise fallback to book/
if [ -d "book/book" ]; then
  MD_OUT="book/book"
else
  MD_OUT="book"
fi

echo "📘 Starting mdBook server (it will build once at start)..."
# Start mdbook serve in background so we can wait for it then copy API docs
# Write logs so we can show them if something goes wrong
MDLOG="$(mktemp)"
mdbook serve book --open >"$MDLOG" 2>&1 &
MDB_PID=$!

# wait for server to respond (max 60s)
echo "⏳ Waiting for mdBook to be ready at $SERVER_URL (will timeout after 60s)..."
TRIES=0
MAX_TRIES=60
until curl -sSf "$SERVER_URL" >/dev/null 2>&1 || [ $TRIES -ge $MAX_TRIES ]; do
  TRIES=$((TRIES+1))
  sleep 1
done

if [ $TRIES -ge $MAX_TRIES ]; then
  echo "❌ mdBook did not become ready within timeout. Dumping mdBook log:"
  echo "------ mdBook log start ------"
  sed -n '1,200p' "$MDLOG"
  echo "------- mdBook log end -------"
  kill $MDB_PID 2>/dev/null || true
  rm -f "$MDLOG"
  exit 1
fi

echo "✅ mdBook is up (took ${TRIES}s). Copying API docs into '$MD_OUT/api/doc'..."

# Remove any old API dir and copy the whole target/doc (preserves search files etc.)
rm -rf "$MD_OUT/api"
mkdir -p "$MD_OUT/api"
cp -r target/doc "$MD_OUT/api/doc"

echo "📂 Copied API docs. API index should be available at:"
echo "   $SERVER_URL/api/doc/ga4ghphetools/index.html"

# Tail mdbook log and wait for the server process — so script keeps running
echo "📖 Tailing mdBook server log (press Ctrl-C to stop)"
tail -n +1 -f "$MDLOG" &
TAIL_PID=$!

# Wait for mdbook to exit; when it does, clean up
wait $MDB_PID || true

# Clean up
kill $TAIL_PID 2>/dev/null || true
rm -f "$MDLOG"
