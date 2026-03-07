## ADDED Requirements

### Requirement: show サブコマンドによる DB 閲覧
システムは SHALL `kabu show <target>` で DB の各テーブル内容を人間が読みやすい形式で表示する。デフォルト出力は human 形式とし、`--format json` で JSON 出力も可能とする。

#### Scenario: ウォッチリスト表示
- **WHEN** `kabu show watchlist` を実行した場合
- **THEN** watchlist テーブルの全銘柄を ticker, 会社名, セクター, ノート, 追加日時とともに表示する

#### Scenario: ウォッチリストイベント表示
- **WHEN** `kabu show events` を実行した場合
- **THEN** watchlist_events テーブルの全イベントを新しい順に表示する

#### Scenario: 特定銘柄のイベント表示
- **WHEN** `kabu show events --ticker 7203` を実行した場合
- **THEN** 指定 ticker のイベントのみをフィルタして表示する

#### Scenario: ポジション表示
- **WHEN** `kabu show positions` を実行した場合
- **THEN** portfolio_positions のアクティブポジションを表示する

#### Scenario: 評価履歴表示
- **WHEN** `kabu show evaluations` を実行した場合
- **THEN** evaluations テーブルの直近20件を新しい順に表示する

#### Scenario: 評価履歴の件数指定
- **WHEN** `kabu show evaluations --limit 5` を実行した場合
- **THEN** 直近5件の評価を表示する

#### Scenario: 登録済み銘柄表示
- **WHEN** `kabu show stocks` を実行した場合
- **THEN** stocks テーブルの全銘柄を表示する

#### Scenario: テーブル統計表示
- **WHEN** `kabu show tables` を実行した場合
- **THEN** 全テーブル名とそれぞれのレコード数を表示する
