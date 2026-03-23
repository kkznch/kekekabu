# Tachibana Securities e-Shiten API Documentation Site Structure

## Page Architecture

The documentation site uses framesets and Shift-JIS encoding.

```
mfds_json_api_refference.html (frameset — do NOT fetch directly)
├── frame[0]: mfds_json_api_title.html      (title bar, skip)
├── frame[1]: mfds_json_api_ref_index.html   (sidebar menu, skip)
└── frame[2]: mfds_json_api_ref_text.html    (★ main content — fetch this)

mfds_json_api_menu.html                      (overview page, JS accordion)
mfds_json_api_compress_v4r8.js               (compress mapping, plain JS)
```

## Key URLs

| Resource | URL | Notes |
|----------|-----|-------|
| API Reference (content) | `https://www.e-shiten.jp/e_api/mfds_json_api_ref_text.html` | Shift-JIS, contains all sCLMID specs |
| Overview/Menu | `https://www.e-shiten.jp/e_api/mfds_json_api_menu.html` | Shift-JIS, JS accordion |
| Compress Mapping | `https://www.e-shiten.jp/e_api/mfds_json_api_compress_v4r8.js` | Shift-JIS, update `v4r8` for newer versions |
| Production Auth | `https://kabuka.e-shiten.jp/e_api_v4r8/auth/` | |
| Demo Auth | `https://demo-kabuka.e-shiten.jp/e_api_v4r8/auth/` | |
| Demo Web UI | `https://demo.e-shiten.jp/` | Standard web interface for demo |

## Encoding

All pages use Shift-JIS encoding. Fetch with:
```bash
curl -s <url> | iconv -f SHIFT_JIS -t UTF-8
```

## agent-browser Limitations

- `snapshot` captures table cells (sCLMID field names) but misses prose sections
- `eval` returns empty string inside iframes (cross-origin restrictions)
- `frame @e3` switches context but text extraction still fails
- **Use `curl + iconv` instead** — reliable and complete

## Content Sections in ref_text.html

The reference page contains these sections (all in one HTML file, toggled by JS):

1. **共通説明** — API URLs (production + demo), access methods, version info
2. **認証機能** — CLMAuthLoginRequest/Ack, CLMAuthLogoutRequest/Ack
3. **業務機能** — Orders (CLMKabuNewOrder, CLMKabuCorrectOrder, CLMKabuCancelOrder), queries (CLMOrderList, CLMOrderListDetail), positions (CLMGenbutuKabuList, CLMShinyouTategyokuList), balances (CLMZanKai*)
4. **マスタ機能** — Master data download (CLMSystemStatus, CLMIssueMst*, etc.)
5. **時価情報機能** — Market prices (CLMMfdsGetMarketPrice*, news, etc.)
6. **EVENT I/F** — WebSocket/HTTP chunk event subscription (CLMEventDownload)
7. **結果コード表** — Error codes and warning codes

## Compress Mapping (Key Compression)

The API uses key compression for JSON payloads. String keys like `"sCLMID"` are replaced with 1-indexed numeric strings like `"334"`.

- Mapping defined in `mfds_json_api_compress_v4r8.js`
- Array `_pa_col` contains 941 sorted column names
- `compress`: key name → binary search → `(index + 1).to_string()`
- `uncompress`: numeric key → `_pa_col[key - 1]`
- Both request and response use compression
- Non-numeric keys pass through unchanged

## Version History Pattern

API versions follow: `e_api_v{major}r{revision}` (e.g., `e_api_v4r8`)
- Compress JS filename includes version: `mfds_json_api_compress_v4r8.js`
- Old versions retired ~60 days after new version release
- Check menu page "リリース＆改定情報" for version announcements
