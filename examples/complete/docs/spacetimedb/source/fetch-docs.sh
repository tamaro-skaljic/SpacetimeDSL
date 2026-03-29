#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

urls=(
  "https://spacetimedb.com/docs/intro/what-is-spacetimedb?server-language=rust"
  "https://spacetimedb.com/docs/intro/zen?server-language=rust"
  "https://spacetimedb.com/docs/intro/key-architecture?server-language=rust"
  "https://spacetimedb.com/docs/intro/faq?server-language=rust"
  "https://spacetimedb.com/docs/tutorials/chat-app/?server-language=rust"
  "https://spacetimedb.com/docs/databases?server-language=rust"
  "https://spacetimedb.com/docs/databases/transactions-atomicity?server-language=rust"
  "https://spacetimedb.com/docs/databases/cheat-sheet?server-language=rust"
  "https://spacetimedb.com/docs/databases/automatic-migrations?server-language=rust"
  "https://spacetimedb.com/docs/databases/incremental-migrations?server-language=rust"
  "https://spacetimedb.com/docs/functions?server-language=rust"
  "https://spacetimedb.com/docs/functions/reducers?server-language=rust"
  "https://spacetimedb.com/docs/functions/reducers/reducer-context?server-language=rust"
  "https://spacetimedb.com/docs/functions/reducers/lifecycle?server-language=rust"
  "https://spacetimedb.com/docs/functions/reducers/error-handling?server-language=rust"
  "https://spacetimedb.com/docs/functions/procedures?server-language=rust"
  "https://spacetimedb.com/docs/functions/views?server-language=rust"
  "https://spacetimedb.com/docs/tables?server-language=rust"
  "https://spacetimedb.com/docs/tables/column-types?server-language=rust"
  "https://spacetimedb.com/docs/tables/file-storage?server-language=rust"
  "https://spacetimedb.com/docs/tables/auto-increment?server-language=rust"
  "https://spacetimedb.com/docs/tables/constraints?server-language=rust"
  "https://spacetimedb.com/docs/tables/default-values?server-language=rust"
  "https://spacetimedb.com/docs/tables/indexes?server-language=rust"
  "https://spacetimedb.com/docs/tables/access-permissions?server-language=rust"
  "https://spacetimedb.com/docs/tables/schedule-tables?server-language=rust"
  "https://spacetimedb.com/docs/tables/event-tables?server-language=rust"
  "https://spacetimedb.com/docs/tables/performance?server-language=rust"
  "https://spacetimedb.com/docs/core-concepts/authentication?server-language=rust"
  "https://spacetimedb.com/docs/core-concepts/authentication/usage?server-language=rust"
  "https://spacetimedb.com/docs/how-to/logging?server-language=rust"
  "https://spacetimedb.com/docs/how-to/reject-client-connections?server-language=rust"
)

for i in "${!urls[@]}"; do
  url="${urls[$i]}"
  num=$((i + 1))
  # Remove the base URL prefix and any trailing slash, then replace / with -
  name="${url#https://spacetimedb.com/docs/}"
  name="${name%/}"
  name="${name//\//-}"
  filename="$(printf '%02d' "$num")-${name}.html"

  echo "Fetching [$num/${#urls[@]}] $url -> $filename"
  curl -sL "$url" -o "$filename"
done

echo "Done. Fetched ${#urls[@]} files."
