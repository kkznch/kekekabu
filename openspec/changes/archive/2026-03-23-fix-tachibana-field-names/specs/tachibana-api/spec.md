## MODIFIED Requirements

### Requirement: CLMKabuNewOrder による株式新規注文
システムは SHALL 公式 v4r8 リファレンス準拠のフィールド名で CLMKabuNewOrder リクエストを構築する。`sSizyouC`, `sBaibaiKubun`, `sGenkinShinyouKubun`, `sCondition`, `sOrderPrice`, `sOrderSuryou`, `sOrderExpireDay`, `sGyakusasiOrderType`, `sGyakusasiZyouken`, `sGyakusasiPrice`, `sTatebiType`, `sZyoutoekiKazeiC`, `sSecondPassword` を使用する。

#### Scenario: 現物買い注文
- **WHEN** Side::Buy で注文を構築した場合
- **THEN** `sBaibaiKubun` = `"3"`, `sGenkinShinyouKubun` = `"0"`, `sZyoutoekiKazeiC` = `"1"`, `sSecondPassword` に第二パスワードを含むリクエスト JSON を生成する

#### Scenario: 現物売り注文
- **WHEN** Side::Sell で注文を構築した場合
- **THEN** `sBaibaiKubun` = `"1"` で、その他は買い注文と同一構造のリクエスト JSON を生成する

#### Scenario: 逆指値フィールドのデフォルト
- **WHEN** 通常の指値注文を構築した場合
- **THEN** `sGyakusasiOrderType` = `"0"`, `sGyakusasiZyouken` = `"0"`, `sGyakusasiPrice` = `"*"` を含む

### Requirement: CLMOrderListDetail レスポンスの公式フィールド名準拠パース
システムは SHALL CLMOrderListDetail レスポンスを公式 v4r8 フィールド名でパースする。

#### Scenario: 注文詳細レスポンスのパース
- **WHEN** CLMOrderListDetail のレスポンスを受信した場合
- **THEN** `sOrderNumber`, `sIssueCode`, `sOrderStatusCode`, `sBaibaiKubun`, `sOrderPrice`, `sOrderSuryou`, `sYakuzyouPrice`, `sYakuzyouSuryou` をパースする
