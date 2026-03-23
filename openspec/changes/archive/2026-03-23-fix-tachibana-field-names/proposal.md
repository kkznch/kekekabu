## Why

立花証券 API v4r8 公式リファレンスとの突合で、CLMKabuNewOrder のフィールド名が複数箇所で不一致であることが判明。特に `sSecondPassword`（第二パスワード）が欠落しており、実発注が不可能。また CLMOrderListDetail のレスポンスパースも実装独自のフィールド名を使っており、デモ環境・本番環境で正しく動作しない。

## What Changes

- CLMKabuNewOrder リクエストのフィールド名を公式に合わせて修正（10箇所）
- `sSecondPassword` を注文リクエストに追加（config から取得）
- `sZyoutoekiKazeiC`（譲渡益課税区分）、逆指値関連フィールドを追加
- CLMOrderListDetail レスポンスパースのフィールド名を公式に合わせて修正
- `build_new_order_json` に `second_password` パラメータを追加
- BrokerClient trait の `place_order` シグネチャに `second_password` を追加

## Capabilities

### New Capabilities

（なし）

### Modified Capabilities

- `tachibana-api`: 注文フィールド名を公式 v4r8 に準拠、sSecondPassword 追加
- `trade-execution`: place_order に second_password を渡す

## Impact

- `src/tachibana/order.rs` — フィールド名修正、sSecondPassword/sZyoutoekiKazeiC 追加
- `src/tachibana/mod.rs` — BrokerClient trait / TachibanaClient に second_password 伝搬
- `src/cmd/execute.rs` — place_order 呼び出し時に second_password を渡す
- `tests/execute_test.rs` — MockBrokerClient の更新
