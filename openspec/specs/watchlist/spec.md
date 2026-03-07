## Purpose

監視銘柄の管理（追加・削除・一覧）。パイプライン全体の対象銘柄を定義し、scan/fetch/eval の起点となる。

## Requirements

### Requirement: ウォッチリストに銘柄を追加
システムは SHALL ticker コードでウォッチリストに銘柄を追加できる。

#### Scenario: 新規銘柄の追加
- **WHEN** `kabu watchlist add 7203` を実行した場合
- **THEN** ticker 7203 をウォッチリストに追加し、確認を stderr にログ出力する

#### Scenario: メモ付きで追加
- **WHEN** `kabu watchlist add 7203 --notes "トヨタ自動車"` を実行した場合
- **THEN** 指定されたメモ付きで銘柄を追加する

#### Scenario: 冪等な追加
- **WHEN** 既にウォッチリストにある銘柄を追加した場合
- **THEN** 重複エントリを作成しない（INSERT OR IGNORE）

### Requirement: ウォッチリストから銘柄を削除
システムは SHALL ticker コードでウォッチリストから銘柄を削除できる。

#### Scenario: 既存銘柄の削除
- **WHEN** `kabu watchlist remove 7203` を実行した場合
- **THEN** ticker 7203 をウォッチリストから削除する

### Requirement: ウォッチリスト銘柄の一覧表示
システムは SHALL ウォッチリストの全銘柄を一覧表示できる。

#### Scenario: JSON 出力での一覧
- **WHEN** `kabu watchlist list` を実行した場合
- **THEN** ウォッチリスト項目を JSON 配列で stdout に出力する（ticker, name, sector, notes）

#### Scenario: human 出力での一覧
- **WHEN** `kabu watchlist list --format human` を実行した場合
- **THEN** ウォッチリスト項目をフォーマットされたテーブルで出力する
