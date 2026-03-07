## Purpose

監視銘柄の管理。discover コマンドが自動で watchlist テーブルを管理する。手動の CLI コマンドは廃止済み。

## Requirements

> 手動の watchlist CLI コマンド（add/remove/list）は discover コマンドに移行済み。
> DB の watchlist テーブルと内部関数（watchlist_add, watchlist_remove, watchlist_list）は discover 等の内部利用として残存。
> 一覧表示は `kabu discover --list` で提供。
