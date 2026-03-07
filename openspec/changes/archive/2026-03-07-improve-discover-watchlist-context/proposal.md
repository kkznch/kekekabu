## Why

discover コマンドは毎回ゼロから銘柄候補を生成しており、現在のウォッチリストを LLM に渡していない。そのため LLM が「意図的に外した」のか「たまたま言及しなかった」のか区別できず、差分管理が機械的になっている。また watchlist の変更履歴が残らないため、銘柄の出入りの経緯を追跡できない。

## What Changes

- discover プロンプトに現在のウォッチリストを渡し、keep/add/remove のアクション別レスポンスを返させる
- LLM の判断に基づいた差分管理に切り替え（現在のコード側での機械的な差分管理を置換）
- `watchlist_events` テーブルを追加し、watchlist の変更履歴（add/remove/keep + 理由）を記録する

## Capabilities

### New Capabilities
- `watchlist-history`: watchlist の変更イベント（add/remove/keep）を記録・追跡する機能

### Modified Capabilities
- `stock-discovery`: discover プロンプトに現在のウォッチリストを含め、keep/add/remove のアクション別レスポンスを返すように変更
- `watchlist`: watchlist_events テーブルの追加と内部関数の拡張

## Impact

- `src/cmd/discover.rs`: プロンプト構築・レスポンスパース・差分管理ロジックの変更
- `src/db/mod.rs` + `src/db/schema.rs`: `watchlist_events` テーブルの追加、イベント記録関数の追加
- 既存のテスト: discover レスポンスパースのテスト更新が必要
