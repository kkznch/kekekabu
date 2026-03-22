#!/bin/bash
# Fetch Tachibana Securities e-Shiten API documentation.
# Handles Shift-JIS encoding and frameset page structure.
#
# Usage:
#   ./fetch_docs.sh ref       # API reference (main content)
#   ./fetch_docs.sh menu      # Overview/menu page
#   ./fetch_docs.sh compress  # Compress mapping JS (key→number mapping)
#   ./fetch_docs.sh all       # All of the above

set -euo pipefail

BASE="https://www.e-shiten.jp/e_api"
OUT_DIR="${2:-.}"

fetch_shift_jis() {
    curl -s "$1" | iconv -f SHIFT_JIS -t UTF-8 || true
}

strip_html() {
    sed 's/<br>/\n/g; s/<[^>]*>//g; s/&nbsp;/ /g; s/&amp;/\&/g; s/&lt;/</g; s/&gt;/>/g' | sed '/^[[:space:]]*$/d'
}

case "${1:-help}" in
    ref)
        # Main API reference — the content frame of the frameset page
        echo "Fetching API reference (ref_text.html)..." >&2
        fetch_shift_jis "${BASE}/mfds_json_api_ref_text.html" | strip_html > "${OUT_DIR}/tachibana_ref_text.txt"
        echo "Saved: ${OUT_DIR}/tachibana_ref_text.txt ($(wc -l < "${OUT_DIR}/tachibana_ref_text.txt") lines)" >&2
        ;;
    menu)
        # Overview/menu page — version info, release notes, setup instructions
        echo "Fetching menu page..." >&2
        fetch_shift_jis "${BASE}/mfds_json_api_menu.html" | strip_html > "${OUT_DIR}/tachibana_menu.txt"
        echo "Saved: ${OUT_DIR}/tachibana_menu.txt ($(wc -l < "${OUT_DIR}/tachibana_menu.txt") lines)" >&2
        ;;
    compress)
        # Compress mapping JS — numeric key ↔ string key mapping (941 items for v4r8)
        echo "Fetching compress JS..." >&2
        fetch_shift_jis "${BASE}/mfds_json_api_compress_v4r8.js" > "${OUT_DIR}/tachibana_compress.js"
        echo "Saved: ${OUT_DIR}/tachibana_compress.js ($(wc -l < "${OUT_DIR}/tachibana_compress.js") lines)" >&2
        ;;
    all)
        "$0" ref "$OUT_DIR"
        "$0" menu "$OUT_DIR"
        "$0" compress "$OUT_DIR"
        ;;
    help|*)
        echo "Usage: $0 {ref|menu|compress|all} [output-dir]"
        echo ""
        echo "  ref       - API reference (sCLMID specs, request/response fields)"
        echo "  menu      - Overview page (version info, release notes, URLs)"
        echo "  compress  - Compress mapping JS (key↔number mapping table)"
        echo "  all       - Fetch all of the above"
        echo ""
        echo "Output dir defaults to current directory."
        ;;
esac
