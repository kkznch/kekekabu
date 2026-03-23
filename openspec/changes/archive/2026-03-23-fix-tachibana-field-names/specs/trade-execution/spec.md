## MODIFIED Requirements

### Requirement: BrokerClient trait 経由の注文発注
システムは SHALL `BrokerClient::place_order` に `second_password` パラメータを追加し、立花証券 API の sSecondPassword フィールドに渡す。

#### Scenario: execute から place_order 呼び出し
- **WHEN** execute コマンドが注文を発注する場合
- **THEN** config の `tachibana.second_password` を `place_order` の `second_password` 引数として渡す

#### Scenario: MockBrokerClient のテスト
- **WHEN** テストで MockBrokerClient を使用する場合
- **THEN** `second_password` 引数を受け取るが検証には使用しない
