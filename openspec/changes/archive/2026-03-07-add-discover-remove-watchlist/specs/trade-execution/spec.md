## MODIFIED Requirements

### Requirement: 当日の評価結果を処理
システムは SHALL 当日の evaluations を処理し、decision とスコアに基づいて売買シグナルを生成する。

#### Scenario: 高スコア Buy の買いシグナル
- **WHEN** evaluation の decision="Buy" かつ score >= 70 の場合
- **THEN** 買いシグナルアクションを生成する

#### Scenario: 低スコア Buy の買いシグナルスキップ
- **WHEN** evaluation の decision="Buy" かつ score < 70 の場合
- **THEN** "score too low" の説明付きで買いシグナルをスキップする

#### Scenario: Sell の売りシグナル
- **WHEN** evaluation の decision="Sell" の場合
- **THEN** portfolio_positions を確認し、保有していれば売りシグナルを生成する。保有していなければスキップする

#### Scenario: 強い Avoid の売りシグナル
- **WHEN** evaluation の decision="Avoid" かつ score <= 30 の場合
- **THEN** 既存ポジションの見直しを促す売りシグナルアクションを生成する

#### Scenario: Hold アクション
- **WHEN** evaluation の decision="Hold" または買い/売りの閾値を満たさない場合
- **THEN** hold アクションを生成する
