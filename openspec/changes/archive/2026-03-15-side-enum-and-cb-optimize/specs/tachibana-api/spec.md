## MODIFIED Requirements

### Requirement: BrokerClient trait による証券 API 抽象化
システムは SHALL `BrokerClient` trait を定義し、`place_order` メソッドの売買方向パラメータとして `Side` enum（Buy/Sell）を使用する。`Side` enum は `as_str()` で `"buy"` / `"sell"` 文字列に変換可能で、`Display` trait を実装する。

#### Scenario: Side enum による型安全な注文
- **WHEN** `BrokerClient::place_order(Side::Buy, ...)` を呼び出した場合
- **THEN** 立花証券 API の `sOrderBaibaiKubun` に `"3"`（買い）が設定される

#### Scenario: Side::Sell の注文
- **WHEN** `BrokerClient::place_order(Side::Sell, ...)` を呼び出した場合
- **THEN** 立花証券 API の `sOrderBaibaiKubun` に `"1"`（売り）が設定される

#### Scenario: Side enum の文字列変換
- **WHEN** `Side::Buy.as_str()` を呼び出した場合
- **THEN** `"buy"` を返す。`Side::Sell.as_str()` は `"sell"` を返す
