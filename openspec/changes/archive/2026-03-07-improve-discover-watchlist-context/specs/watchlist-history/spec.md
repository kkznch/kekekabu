## ADDED Requirements

### Requirement: watchlist 変更イベントの記録
システムは SHALL discover 実行時に watchlist への変更（add/remove/keep）を `watchlist_events` テーブルに記録する。各イベントには ticker、アクション種別、LLM が返した理由、discover 実行日時を含める。

#### Scenario: 銘柄追加イベントの記録
- **WHEN** discover が新規銘柄を watchlist に追加した場合
- **THEN** action="add"、LLM が返した理由を reason に設定して `watchlist_events` に記録する

#### Scenario: 銘柄削除イベントの記録
- **WHEN** discover が既存銘柄を watchlist から削除した場合
- **THEN** action="remove"、LLM が返した理由を reason に設定して `watchlist_events` に記録する

#### Scenario: 銘柄維持イベントの記録
- **WHEN** discover が既存銘柄を watchlist に維持すると判断した場合
- **THEN** action="keep"、LLM が返した理由を reason に設定して `watchlist_events` に記録する

### Requirement: watchlist_events テーブルスキーマ
システムは SHALL `watchlist_events` テーブルを以下のカラムで作成する: id, ticker, action (add/remove/keep), reason, discovered_at。

#### Scenario: テーブル作成
- **WHEN** アプリケーション起動時にデータベースを初期化する場合
- **THEN** `watchlist_events` テーブルが存在しなければ作成する
