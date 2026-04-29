## ADDED Requirements

### Requirement: kabu sync コマンドで証券口座と DB を同期
システムは SHALL `kabu sync` コマンドを提供し、立花証券 API から実残高・実ポジションを取得して DB と突合する。

#### Scenario: 残高取得と保存
- **WHEN** `kabu sync` を実行した場合
- **THEN** 立花証券 API の `CLMZanKaiKanougaku` を呼び出し、買付可能額を取得して `account_balance` テーブルに新規行として保存する

#### Scenario: ポジション突合
- **WHEN** `kabu sync` を実行した場合
- **THEN** 立花証券 API の `CLMGenbutuKabuList` を呼び出し、現物保有銘柄を取得し、DB の `portfolio_positions` と数量を比較する

#### Scenario: 不整合検出
- **WHEN** DB 上の数量と証券口座の数量が異なる銘柄がある場合
- **THEN** 不整合の一覧（ticker, DB 数量, 実数量, 差分）を warn ログで出力し、stdout に JSON または human 形式で表示する

#### Scenario: --fix フラグでの自動補正
- **WHEN** `kabu sync --fix` を実行し、不整合がある場合
- **THEN** DB の `portfolio_positions.quantity` を実建玉の数量に合わせて更新する。新規ポジション（DB にない銘柄）は INSERT、削除ポジション（実建玉にない銘柄）は DELETE する。`avg_cost` は変更しない

#### Scenario: --fix なしの場合は読み取り専用
- **WHEN** `kabu sync` を `--fix` なしで実行した場合
- **THEN** 不整合があっても DB は変更せず、ログと出力のみで通知する

### Requirement: --demo フラグで sync も対応
システムは SHALL `kabu --demo sync` でデモ環境の口座同期を可能にする。

#### Scenario: デモ環境での同期
- **WHEN** `kabu --demo sync` を実行した場合
- **THEN** デモ環境の `demo-kabuka.e-shiten.jp` に接続し、デモ DB（`kekekabu-demo.db`）に対して同期を実行する

### Requirement: sync の認証と接続管理
システムは SHALL sync 実行時に立花証券 API へログインし、終了時にログアウトする。

#### Scenario: 認証エラー
- **WHEN** ログインに失敗した場合
- **THEN** エラーログを出力して異常終了する。DB は変更しない

#### Scenario: 終了時のログアウト
- **WHEN** sync 処理が完了した場合（成功・失敗いずれでも）
- **THEN** 立花証券 API へログアウトリクエストを送信する
