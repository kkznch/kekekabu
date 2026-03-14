## MODIFIED Requirements

### Requirement: Avoid 判定銘柄の処理
システムは SHALL Avoid 判定の銘柄に対してレビューアクション（action_type="review"）を記録する。

#### Scenario: Avoid 銘柄のアクション記録
- **WHEN** eval が Avoid 判定を返した場合
- **THEN** action_type="review" として記録し、注文は発行しない

### Requirement: サーキットブレーカー発動時の処理
システムは SHALL サーキットブレーカー発動時に安全にセッションを終了する。

#### Scenario: CB 発動時の logout
- **WHEN** サーキットブレーカーが発動した場合
- **THEN** 全注文をスキップし、Tachibana API からログアウトしてからエラーを返す

### Requirement: settle フェーズでの約定確認
システムは SHALL settle フェーズで未決済注文（pending または partial）の約定状況を照会し、ステータスとポートフォリオをアトミックに更新する。

#### Scenario: 全部約定の反映
- **WHEN** settle で pending 注文が全部約定していた場合
- **THEN** 1つのトランザクション内で注文ステータスを "filled" に更新し、ポートフォリオに約定を記録する

#### Scenario: 一部約定の反映
- **WHEN** settle で pending 注文が一部約定していた場合
- **THEN** 1つのトランザクション内で注文ステータスを "partial" に更新し、ポートフォリオに約定分を記録する

#### Scenario: 非約定ステータスの更新
- **WHEN** settle で注文が expired または rejected の場合
- **THEN** 注文ステータスのみを更新し、ポートフォリオは変更しない
