## MODIFIED Requirements

### Requirement: 売買シグナルの生成と発注
システムは SHALL execute コマンド内のシグナル（Signal 構造体）および注文結果（OrderResult 構造体）の売買方向フィールドとして `Side` enum を使用する。文字列リテラル `"buy"` / `"sell"` の直接使用を排除する。

#### Scenario: Buy シグナルの生成
- **WHEN** eval の判断が Buy で、冪等性チェック（order_exists_for_evaluation）を通過した場合
- **THEN** `Signal { side: Side::Buy, ... }` を生成する

#### Scenario: Sell シグナルの生成
- **WHEN** eval の判断が Sell、またはハードストップロスが発動した場合
- **THEN** `Signal { side: Side::Sell, ... }` を生成する

#### Scenario: DB 保存時の文字列変換
- **WHEN** Signal を DB に保存する場合（save_order, order_exists_for_evaluation）
- **THEN** `side.as_str()` で `"buy"` / `"sell"` に変換して保存する
