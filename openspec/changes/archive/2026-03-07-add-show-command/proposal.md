## Why

DB の中身を確認するコマンドが散らばっており（`discover --list`, `portfolio positions`, `history`）、watchlist_events に至っては表示手段がない。`kabu show` で DB の各テーブルを見やすく一覧表示できるサブコマンドが必要。

## What Changes

- `kabu show` サブコマンドを追加（watchlist / events / positions / evaluations / stocks / tables）
- 既存の `kabu discover --list` を `kabu show watchlist` に統合し、`--list` フラグを廃止
- 既存の `kabu history` を `kabu show evaluations` に統合し、`history` サブコマンドを廃止
- human 出力をデフォルトとし、`--format json` で JSON 出力も可能

## Capabilities

### New Capabilities
- `show`: DB 内容の閲覧用サブコマンド

### Modified Capabilities
- `stock-discovery`: `discover --list` を廃止し `show watchlist` に移行
- `database`: テーブル数を 7 → 8 に更新（watchlist_events 追加済みだが未反映）

## Impact

- `src/main.rs`: `Show` サブコマンド追加、`History` 削除、`Discover` の `--list` 削除
- `src/cmd/show.rs`: 新規ファイル
- `src/cmd/mod.rs`: `show` モジュール追加
- `src/db/mod.rs`: watchlist_events 取得関数追加
- `src/output.rs`: show 用の HumanDisplay 実装追加
- `README.md`: コマンド一覧の更新
- `CLAUDE.md`: コマンド一覧の更新
