## ADDED Requirements

### Requirement: BrokerClient trait の残高・建玉照会メソッド
システムは SHALL `BrokerClient` trait に `query_balance()` と `query_positions()` メソッドを追加し、立花証券 API の `CLMZanKaiKanougaku` と `CLMGenbutuKabuList` を呼び出して結果を構造化された Rust 型で返す。

#### Scenario: 買付可能額の取得
- **WHEN** `BrokerClient::query_balance()` を呼び出した場合
- **THEN** 立花証券 API の `CLMZanKaiKanougaku` を仮想 URL（REQUEST）に圧縮 POST し、レスポンスを展開して `BrokerBalance { cash_available: Decimal }` を返す

#### Scenario: 現物保有銘柄の取得
- **WHEN** `BrokerClient::query_positions()` を呼び出した場合
- **THEN** 立花証券 API の `CLMGenbutuKabuList` を仮想 URL（REQUEST）に圧縮 POST し、レスポンスを展開して `Vec<BrokerPosition>`（各要素は ticker, quantity, avg_cost）を返す

#### Scenario: API エラー時の伝播
- **WHEN** API がエラーレスポンス（`p_errno` が 0 以外）を返した場合
- **THEN** `query_balance()` / `query_positions()` は `Result::Err` でエラー詳細を返す
