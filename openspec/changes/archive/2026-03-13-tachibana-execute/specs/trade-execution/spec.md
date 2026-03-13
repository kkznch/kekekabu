## ADDED Requirements

### Requirement: settle フェーズによる前回注文の約定確認
システムは SHALL execute 冒頭で orders テーブルの pending 注文を立花証券 API で照会し、約定済みなら portfolio に記録する。

#### Scenario: 約定済み注文の settle
- **WHEN** pending 注文の sOrderStatusCode が "10"（全部約定）の場合
- **THEN** portfolio::buy または portfolio::sell を呼び出してポジション・取引履歴を更新し、orders.status を "filled" に更新する

#### Scenario: 失効注文の settle
- **WHEN** pending 注文の sOrderStatusCode が "12"（全部失効）の場合
- **THEN** orders.status を "expired" に更新し、portfolio は変更しない

#### Scenario: まだ未約定の注文
- **WHEN** pending 注文の sOrderStatusCode が "1"（未約定）の場合
- **THEN** orders.status は "pending" のまま残し、次回の settle で再確認する

#### Scenario: settle 対象がない場合
- **WHEN** pending 注文が存在しない場合
- **THEN** 立花 API にログインせず settle フェーズをスキップする

### Requirement: 実注文の発注
システムは SHALL dry-run でない場合にシグナルに基づいて立花証券 API 経由で指値注文を発注する。

#### Scenario: Buy シグナルからの発注
- **WHEN** Buy シグナル（score >= 70）が生成され dry-run でない場合
- **THEN** 立花 API で指値買い注文を発注し、orders テーブルに pending で記録する

#### Scenario: Sell シグナルからの発注
- **WHEN** Sell シグナルが生成され保有ポジションがあり dry-run でない場合
- **THEN** 立花 API で指値売り注文を発注し、orders テーブルに pending で記録する

#### Scenario: dry-run モードでの発注スキップ
- **WHEN** dry-run モードで実行した場合
- **THEN** 立花 API には接続せず、従来どおり "[DRY RUN]" プレフィックス付きシグナルを出力する

#### Scenario: 同一評価からの重複発注防止
- **WHEN** 同じ evaluation から既に注文が存在する場合
- **THEN** 重複発注をスキップし、ログに既存注文の存在を記録する

### Requirement: 短時間 WebSocket 約定待ち
システムは SHALL 発注後に EVENT I/F WebSocket に接続し、設定されたタイムアウト時間まで約定通知を待機する。

#### Scenario: 即座の約定
- **WHEN** 発注後に WebSocket で約定通知を受信した場合
- **THEN** settle と同様に portfolio に記録し、orders.status を "filled" に更新する

#### Scenario: タイムアウト
- **WHEN** 設定されたタイムアウト時間内に約定通知が受信されなかった場合
- **THEN** WebSocket を切断し、orders は pending のまま残す（翌日の settle で確認）

#### Scenario: WebSocket 接続失敗
- **WHEN** WebSocket 接続に失敗した場合
- **THEN** 警告ログを出力し、約定待ちフェーズをスキップする（注文自体は有効）

## MODIFIED Requirements

### Requirement: ドライランのサポート
システムは SHALL デフォルトでドライランモードとし、アクションに "[DRY RUN]" プレフィックスを付ける。dry-run でない場合は立花証券 API 経由で実注文を発注する。

#### Scenario: ドライランモード
- **WHEN** `kabu execute --dry-run` を実行した場合
- **THEN** 立花証券 API に接続せず、"[DRY RUN]" プレフィックス付きでアクションを出力する

#### Scenario: 実行モード
- **WHEN** `kabu execute` を dry-run なしで実行した場合
- **THEN** 立花証券 API に接続し、シグナルに基づいて実際の注文を発注する
