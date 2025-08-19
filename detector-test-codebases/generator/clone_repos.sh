#!/usr/bin/env bash
# Simple repo cloner: read a file with one GitHub repo URL per line and clone into the target dir
# Usage: ./clone_repos.sh repos.txt output_dir
set -euo pipefail
if [ "$#" -lt 2 ]; then
  echo "Usage: $0 repos.txt output_dir"
  exit 2
fi
repos_file="$1"
outdir="$2"
mkdir -p "$outdir"
while IFS= read -r repo || [ -n "$repo" ]; do
  repo_trim=$(echo "$repo" | sed -e 's/#.*//' -e 's/^\s*//' -e 's/\s*$//')
  [ -z "$repo_trim" ] && continue
  echo "Cloning $repo_trim"
  git clone --depth 1 "$repo_trim" "$outdir/$(basename "$repo_trim" .git)" || echo "Failed: $repo_trim"
done < "$repos_file"
