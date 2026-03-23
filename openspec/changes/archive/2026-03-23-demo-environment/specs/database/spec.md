## MODIFIED Requirements

### Requirement: SQLite を使った10テーブル構成のデータベース
システムは SHALL `db_path()` 関数に `Environment` 引数を受け取り、本番環境では `~/.config/kabu/kekekabu.db`、デモ環境では `~/.config/kabu/kekekabu-demo.db` を返す。

#### Scenario: 本番環境の DB パス
- **WHEN** `db_path(Environment::Production)` を呼び出した場合
- **THEN** `~/.config/kabu/kekekabu.db` を返す

#### Scenario: デモ環境の DB パス
- **WHEN** `db_path(Environment::Demo)` を呼び出した場合
- **THEN** `~/.config/kabu/kekekabu-demo.db` を返す
