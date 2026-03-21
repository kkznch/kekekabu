## Purpose

サーキットブレーカーによる安全制御。異常相場（個別銘柄の急変動・市場全体の急落）を検知し、自動売買をブロックする。

## Requirements

### Requirement: サーキットブレーカーによる異常検知
システムは SHALL サーキットブレーカーの価格変動チェックにおいて、全価格履歴ではなく直近2件の終値のみを取得して判定する。`DbClient` trait に `get_latest_closes(stock_id, n)` メソッドを追加し、circuit_breaker はこれを使用する。ウォッチリスト内の銘柄が1日で30%以上変動した場合、売買実行をブロックする。

#### Scenario: 直近終値による異常判定
- **WHEN** circuit_breaker が銘柄の価格変動をチェックする場合
- **THEN** `get_latest_closes(stock_id, 2)` で直近2件の終値を取得し、変動率を算出する。全価格履歴（fetch_price_data）は使用しない

#### Scenario: 価格データ不足
- **WHEN** 直近終値が2件未満の場合
- **THEN** その銘柄の異常判定をスキップする

#### Scenario: 個別銘柄のサーキットブレーカー
- **WHEN** ウォッチリスト内の銘柄の日次変動率が30%を超えた場合
- **THEN** サーキットブレーカーを発動し、execute を中止して理由を報告する

### Requirement: 市場全体の急落でサーキットブレーカーを発動
システムは SHALL ウォッチリスト銘柄の50%以上が5%以上下落した場合、売買実行をブロックする。

#### Scenario: 市場全体のサーキットブレーカー
- **WHEN** ウォッチリスト銘柄の50%超が日次5%以上の下落を示した場合
- **THEN** サーキットブレーカーを発動し、execute を中止して理由を報告する

### Requirement: サーキットブレーカーの理由報告
システムは SHALL サーキットブレーカー発動のすべての理由を execute 出力に含める。

#### Scenario: 複数のトリガー
- **WHEN** 個別銘柄・市場全体の両方の閾値を超えた場合
- **THEN** すべてのトリガー理由を `circuit_breaker_reasons` 配列に含める

### Requirement: execute の明示的実行モード
システムは SHALL `execute` コマンドで `--dry-run` か `--live` のいずれかのフラグを明示的に指定することを必須とし、フラグなしの場合はヘルプを表示して終了する。

#### Scenario: フラグ未指定
- **WHEN** `kabu execute` をフラグなしで実行した場合
- **THEN** ヘルプメッセージを表示して終了する（注文は発行されない）

#### Scenario: ドライラン
- **WHEN** `kabu execute --dry-run` を実行した場合
- **THEN** ドライランモードで動作する（実際の注文は発行されない）

#### Scenario: 本番実行
- **WHEN** `kabu execute --live` を実行した場合
- **THEN** 立花証券 API 経由で実際の注文を発行する
