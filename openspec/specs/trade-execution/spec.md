## Purpose

評価結果に基づく売買シグナル出力。サーキットブレーカーで安全確認後に、decision とスコアから売買アクションを生成する。

## Requirements

### Requirement: 当日の評価結果を処理
システムは SHALL 当日の evaluations を処理し、decision とスコアに基づいて売買シグナルを生成する。

#### Scenario: 高スコア Buy の買いシグナル
- **WHEN** evaluation の decision="Buy" かつ score >= 70 の場合
- **THEN** 買いシグナルアクションを生成する

#### Scenario: 低スコア Buy の買いシグナルスキップ
- **WHEN** evaluation の decision="Buy" かつ score < 70 の場合
- **THEN** "score too low" の説明付きで買いシグナルをスキップする

#### Scenario: 強い Avoid の売りシグナル
- **WHEN** evaluation の decision="Avoid" かつ score <= 30 の場合
- **THEN** 既存ポジションの見直しを促す売りシグナルアクションを生成する

#### Scenario: Hold アクション
- **WHEN** evaluation の decision="Hold" または買い/売りの閾値を満たさない場合
- **THEN** hold アクションを生成する

### Requirement: ドライランのサポート
システムは SHALL デフォルトでドライランモードとし、アクションに "[DRY RUN]" プレフィックスを付ける。

#### Scenario: ドライランモード
- **WHEN** `kabu execute --dry-run true` を実行した場合
- **THEN** 実際の注文を発行せず、"[DRY RUN]" プレフィックス付きでアクションを出力する

### Requirement: 処理前にサーキットブレーカーを確認
システムは SHALL evaluations の処理前にサーキットブレーカーを確認する。

#### Scenario: サーキットブレーカー発動
- **WHEN** サーキットブレーカーが危険な市場状況を検知した場合
- **THEN** `circuit_breaker_triggered: true` と理由一覧を返して execute を中止する

#### Scenario: 当日の評価がない場合
- **WHEN** 当日の evaluations が存在しない場合
- **THEN** 空のアクションと情報ログメッセージを返す
