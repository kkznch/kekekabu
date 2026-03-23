---
name: tachibana-docs
description: Fetch and parse Tachibana Securities e-Shiten API documentation. Use when checking API specs, updating API docs, investigating sCLMID fields, verifying compress key mappings, or working with the Tachibana Securities broker API. Triggers on mentions of "tachibana docs", "e-shiten API", "立花証券 API ドキュメント", "sCLMID", or when updating docs/tachibana-api/ files.
---

# Tachibana Securities API Documentation Fetcher

Fetch and parse documentation from the Tachibana Securities e-Shiten API site, which uses Shift-JIS encoding and frameset pages.

## Quick Start

Fetch all documentation to a temp directory:

```bash
./scripts/fetch_docs.sh all /tmp
```

This creates:
- `/tmp/tachibana_ref_text.txt` — Full API reference (sCLMID specs, fields, examples)
- `/tmp/tachibana_menu.txt` — Overview, version info, release notes
- `/tmp/tachibana_compress.js` — Key compression mapping (941 items)

## Fetching Individual Resources

```bash
./scripts/fetch_docs.sh ref /tmp       # API reference only
./scripts/fetch_docs.sh menu /tmp      # Overview/menu only
./scripts/fetch_docs.sh compress /tmp  # Compress mapping JS only
```

## Important: Do NOT Use agent-browser

The site uses framesets and Shift-JIS encoding. `agent-browser snapshot` captures only table cells, and `eval` returns empty inside iframes. **Always use the fetch script** (`curl` + `iconv`) for reliable extraction.

See [references/site-structure.md](references/site-structure.md) for the full site architecture, URL list, and compress mapping details.

## Common Workflows

### Update API reference docs

1. Run `./scripts/fetch_docs.sh ref /tmp`
2. Read `/tmp/tachibana_ref_text.txt`
3. Compare with `docs/tachibana-api/api_reference_v4r8.md`
4. Update docs as needed

### Check compress key mapping

1. Run `./scripts/fetch_docs.sh compress /tmp`
2. Extract `_pa_col` array from `/tmp/tachibana_compress.js`
3. Compare with `src/tachibana/compress.rs` COLUMNS array

### Check for new API version

1. Run `./scripts/fetch_docs.sh menu /tmp`
2. Search for "リリース＆改定情報" in output
3. If new version found, update URLs in fetch script and codebase
