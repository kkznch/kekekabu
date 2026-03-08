## MODIFIED Requirements

> 手動の watchlist CLI コマンド（add/remove/list）は discover コマンドに移行済み。
> DB の watchlist テーブルと内部関数（watchlist_add, watchlist_remove, watchlist_list）は discover 等の内部利用、および売却時の自動除外として残存。
> 一覧表示は `kabu show watchlist` で提供。
> watchlist_events テーブルで変更履歴を記録（discover による add/remove/keep、および売却による auto-removed-on-sell）。
