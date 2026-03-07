## Purpose

投資資金の管理と残高計算。Spec の initial_cash と DB の取引履歴から投資可能額を算出し、LLM プロンプトに Budget Context として注入する。

## Requirements

### Requirement: Spec TOML の budget セクションから初期資金を読み取る
システムは SHALL Spec TOML の `[budget]` セクションから `initial_cash` を読み取り、投資可能額の計算に使用する。

#### Scenario: budget セクションが存在する場合
- **WHEN** Spec TOML に `[budget]` セクションと `initial_cash` フィールドが存在する場合
- **THEN** initial_cash の値を数値として取得する

#### Scenario: budget セクションが存在しない場合
- **WHEN** Spec TOML に `[budget]` セクションが存在しない場合
- **THEN** Budget Context の生成をスキップする（エラーにしない）

### Requirement: trades テーブルから投資済み額と回収額を集計する
システムは SHALL trades テーブルの売買履歴から、買い約定の合計額と売り約定の合計額を集計する。

#### Scenario: 取引履歴がある場合
- **WHEN** trades テーブルに buy/sell の履歴が存在する場合
- **THEN** `total_invested = Σ(buy の quantity × price)` と `total_recovered = Σ(sell の quantity × price)` を算出する

#### Scenario: 取引履歴がない場合
- **WHEN** trades テーブルが空の場合
- **THEN** total_invested = 0, total_recovered = 0 とする

### Requirement: 残り投資可能額を算出する
システムは SHALL `remaining = initial_cash - total_invested + total_recovered` で残り投資可能額を計算する。

#### Scenario: 残り投資可能額の算出
- **WHEN** initial_cash = 300,000, total_invested = 120,000, total_recovered = 30,000 の場合
- **THEN** remaining = 210,000 を返す

### Requirement: Budget Context をプロンプト用テキストとして生成する
システムは SHALL Budget Context を LLM プロンプトに注入可能なテキストとして生成する。

#### Scenario: Budget Context の生成
- **WHEN** initial_cash, total_invested, total_recovered, remaining, 保有銘柄数が算出済みの場合
- **THEN** 「初期資金」「投資済み」「回収済み」「残り投資可能額」「保有銘柄数」を含むテキストを生成する
