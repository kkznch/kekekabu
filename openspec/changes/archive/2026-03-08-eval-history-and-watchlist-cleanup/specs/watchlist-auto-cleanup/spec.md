## ADDED Requirements

### Requirement: 売却によるポジションクローズ時に watchlist から自動除外
システムは SHALL ポートフォリオのポジションが売却により残数ゼロになった場合、当該銘柄を watchlist から自動的に除外し、watchlist_events に action="auto-removed-on-sell" のイベントを記録する。

#### Scenario: 全株売却でポジションクローズ
- **WHEN** 100株保有している銘柄の100株を売却した場合
- **THEN** ポジションを is_active=0 に設定し、watchlist から削除し、watchlist_events に auto-removed-on-sell を記録する

#### Scenario: 部分売却ではポジション維持
- **WHEN** 100株保有している銘柄の50株を売却した場合
- **THEN** ポジションを quantity=50 に更新するが、watchlist からは削除しない

#### Scenario: watchlist に存在しない銘柄のポジションクローズ
- **WHEN** watchlist に登録されていない銘柄のポジションが売却でクローズされた場合
- **THEN** watchlist 削除はスキップし（削除対象がない）、エラーにはならない
