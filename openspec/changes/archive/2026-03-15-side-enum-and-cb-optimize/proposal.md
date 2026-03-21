## Why

execute / order / DB 層で売買方向（buy/sell）が文字列リテラルで散在しており、タイポや大文字小文字の不一致による誤発注リスクがある。また circuit_breaker が watchlist 全銘柄の全価格履歴を N+1 パターンで取得しており、銘柄数に比例して不要なデータ読み込みが発生している。

## What Changes

- `Side` enum（Buy/Sell）を導入し、execute・order・BrokerClient・DB 層の全 `"buy"`/`"sell"` 文字列を型安全に置換
- circuit_breaker の N+1 クエリを最適化：全価格履歴取得 → 直近2件の終値のみ取得に変更

## Capabilities

### New Capabilities
（なし）

### Modified Capabilities
- `tachibana-api`: `BrokerClient` trait の `place_order` が `side: &str` → `side: Side` に変更
- `trade-execution`: execute 内部の Signal / OrderResult が `side: String` → `side: Side` に変更
- `safety`: circuit_breaker が全価格履歴ではなく直近終値のみで判定するよう最適化

## Impact

- `src/tachibana/mod.rs` — `Side` enum 定義、`BrokerClient` trait 変更
- `src/tachibana/order.rs` — `build_new_order_json` が `Side` を受け取る
- `src/cmd/execute.rs` — Signal / OrderResult の side フィールド、全 `"buy"`/`"sell"` リテラル置換
- `src/circuit_breaker.rs` — `fetch_price_data` → `get_latest_closes` に変更
- `src/db/mod.rs` — `DbClient` trait に `get_latest_closes` メソッド追加
- `tests/execute_test.rs` — MockBrokerClient の `place_order` シグネチャ変更
