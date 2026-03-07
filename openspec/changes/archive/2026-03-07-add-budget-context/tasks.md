## 1. Budget 集計クエリ

- [x] 1.1 `db/mod.rs` に trades テーブルから買い合計額・売り合計額を集計する関数 `trade_cash_summary` を追加
- [x] 1.2 集計関数のテスト追加（取引なし / 買いのみ / 買い+売り）

## 2. Spec から budget 情報の抽出

- [x] 2.1 `spec.rs` の `InvestmentSpec` に `budget_initial_cash() -> Option<f64>` メソッドを追加（toml::Table から `budget.initial_cash` を取得）
- [x] 2.2 budget 抽出のテスト追加（budget あり / budget なし）

## 3. Budget Context 生成

- [x] 3.1 Budget Context テキストを生成する関数を追加（initial_cash, invested, recovered, remaining, 保有数を受け取りフォーマット済みテキストを返す）
- [x] 3.2 Budget Context 生成のテスト追加

## 4. discover プロンプトへの注入

- [x] 4.1 `cmd/discover.rs` の `run` で Budget Context を組み立て、`build_discover_prompt` に渡す
- [x] 4.2 `build_discover_prompt` に budget_context パラメータを追加し、プロンプトに Budget Context セクションを注入

## 5. eval プロンプトへの注入

- [x] 5.1 `cmd/eval.rs` で Budget Context を組み立て、eval プロンプトに注入

## 6. ユーザー向け Spec 更新

- [x] 6.1 ユーザーの Spec TOML (`jp-core-value-quality-v1.toml`) に `[budget]` セクションを追加
- [x] 6.2 README の Spec 説明に budget セクションの記述を追加

## 7. 検証

- [x] 7.1 `cargo test` 全テスト通過を確認
- [x] 7.2 `cargo clippy` 警告なしを確認
