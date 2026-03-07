## ADDED Requirements

### Requirement: watchlist_events の取得
システムは SHALL watchlist_events テーブルからイベント一覧を取得する関数を提供する。オプションで ticker によるフィルタリングが可能。

#### Scenario: 全イベント取得
- **WHEN** ticker 指定なしで取得した場合
- **THEN** 全イベントを discovered_at の降順で返す

#### Scenario: ticker 指定イベント取得
- **WHEN** ticker を指定して取得した場合
- **THEN** 指定 ticker のイベントのみを discovered_at の降順で返す

### Requirement: テーブル統計の取得
システムは SHALL 全テーブルのレコード数を取得する関数を提供する。

#### Scenario: テーブル統計取得
- **WHEN** テーブル統計を取得した場合
- **THEN** 各テーブル名とレコード数のペアのリストを返す
