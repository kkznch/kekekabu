## 1. CLMKabuNewOrder フィールド名修正

- [x] 1.1 `order.rs` の `build_new_order_json` のフィールド名を公式 v4r8 に修正（sOrderSizyouC→sSizyouC, sOrderBaibaiKubun→sBaibaiKubun, sGenkinSinyouKubun→sGenkinShinyouKubun, sOrderCondition→sCondition, sOrderOrderPrice→sOrderPrice, sOrderOrderSuryou→sOrderSuryou, sOrderOrderExpireDay→sOrderExpireDay, sOrderTatebiType→sTatebiType）
- [x] 1.2 `sOrderOrderPriceKubun` を削除（公式に存在しないフィールド）
- [x] 1.3 `sZyoutoekiKazeiC` = `"1"`（特定口座）を追加
- [x] 1.4 `sGyakusasiOrderType` = `"0"`, `sGyakusasiZyouken` = `"0"`, `sGyakusasiPrice` = `"*"` を追加
- [x] 1.5 `sSecondPassword` パラメータを `build_new_order_json` に追加

## 2. BrokerClient trait 修正

- [x] 2.1 `BrokerClient::place_order` に `second_password: &str` を追加
- [x] 2.2 `TachibanaClient::place_order` で config の second_password を渡す
- [x] 2.3 `cmd/execute.rs` の place_order 呼び出しに second_password を渡す

## 3. CLMOrderListDetail レスポンスフィールド修正

- [x] 3.1 `parse_order_detail_value` のフィールド名を公式に修正（sOrderBaibaiKubun→sBaibaiKubun, sOrderOrderPrice→sOrderPrice, sOrderOrderSuryou→sOrderSuryou）

## 4. テスト更新

- [x] 4.1 `order.rs` のテストを新フィールド名に更新
- [x] 4.2 `execute_test.rs` の MockBrokerClient に second_password を追加
- [x] 4.3 `just ci` で全テスト通過確認
