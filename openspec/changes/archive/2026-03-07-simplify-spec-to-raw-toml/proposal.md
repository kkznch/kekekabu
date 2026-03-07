## Why

投資 Spec の TOML を型付き構造体（`InvestmentSpec { universe, scoring, execution }`）でパースしているが、実際の TOML ファイルの構造と乖離しており、`serde(default)` のせいで未知フィールドが無視されてバリデーションが通ってしまう。Spec は LLM プロンプトに埋め込むだけで Rust 側で数値を使ったフィルタリングはしていないため、型付きパースは不要。

## What Changes

- `InvestmentSpec` 構造体を `name: String` + `raw_content: String` のみに簡素化
- `to_prompt_section()` を生 TOML テキストをそのまま返す実装に変更
- `validate()` メソッドを削除し、TOML 構文チェック + `name` キー存在確認のみに
- 不要になった `UniverseFilter`, `ScoringConfig`, `ScoringFactor`, `ExecutionConfig` 構造体を削除
- テストを新方式に合わせて書き直し
- README の投資 Spec 説明を更新（構造は自由、`name` のみ必須）

## Capabilities

### New Capabilities

(なし)

### Modified Capabilities

- `config`: バリデーション内容の変更（型付きバリデーション → TOML 構文 + name 存在チェック）

## Impact

- `src/spec.rs` — 全面書き換え（構造体削減、パース方式変更）
- `src/cmd/config.rs` — 影響なし（`s.name` のみ使用）
- `src/cmd/eval.rs`, `src/cmd/discover.rs` — 影響なし（`to_prompt_section()` の戻り値型は同じ）
- config テンプレート（`SPEC_TEMPLATE`）— 影響なし（`name` フィールドは元からある）
- README.md — 投資 Spec セクションの記述更新
