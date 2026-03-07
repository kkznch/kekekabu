## REMOVED Requirements

### Requirement: ウォッチリストに銘柄を追加
**Reason**: discover コマンドが自動で watchlist を管理するため、手動追加の CLI コマンドは不要になった。
**Migration**: `kabu discover` が自動で銘柄を追加する。DB の watchlist テーブルと `db::watchlist_add` 関数は内部利用として残る。

### Requirement: ウォッチリストから銘柄を削除
**Reason**: discover コマンドが自動で watchlist を管理するため、手動削除の CLI コマンドは不要になった。
**Migration**: `kabu discover` が差分管理で自動削除する。

### Requirement: ウォッチリスト銘柄の一覧表示
**Reason**: `kabu discover --list` に移行。
**Migration**: `kabu discover --list` で同等の機能を提供する。
