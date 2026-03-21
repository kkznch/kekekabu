## 1. Side enum 導入

- [x] 1.1 `Side` enum（Buy/Sell）を `tachibana/mod.rs` に定義（as_str, Display 実装）
- [x] 1.2 `BrokerClient` trait の `place_order` を `side: Side` に変更
- [x] 1.3 `TachibanaClient::place_order` と trait impl を `Side` に対応
- [x] 1.4 `order.rs` の `build_new_order_json` を `Side` に変更
- [x] 1.5 `execute.rs` の Signal / OrderResult の side フィールドを `Side` に変更
- [x] 1.6 `execute.rs` 内の全 `"buy"` / `"sell"` リテラルを `Side::Buy` / `Side::Sell` に置換
- [x] 1.7 DB 保存箇所で `side.as_str()` を使用するよう変更
- [x] 1.8 `tests/execute_test.rs` の MockBrokerClient を `Side` に対応

## 2. circuit_breaker N+1 最適化

- [x] 2.1 `DbClient` trait に `get_latest_closes(stock_id: i64, n: usize) -> Result<Vec<f64>>` を追加
- [x] 2.2 `SqliteClient` で `get_latest_closes` を実装（ORDER BY date DESC LIMIT n）
- [x] 2.3 `circuit_breaker.rs` を `fetch_price_data` → `get_latest_closes` に変更
- [x] 2.4 `tests/db_test.rs` に `get_latest_closes` のテストを追加

## 3. 検証

- [x] 3.1 `just ci` で全テスト通過を確認
