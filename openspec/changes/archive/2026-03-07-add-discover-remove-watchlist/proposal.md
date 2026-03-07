## Why

現在のパイプライン（scan → fetch → eval → execute → report）では、対象銘柄を人が手動で `kabu watchlist add` する必要がある。自動売買ツールとしては、銘柄発掘も LLM に任せて完全自律パイプライン（discover → scan → fetch → eval → execute → report）にすべき。また、eval は新規候補（Hunting）と保有継続（Farming）の2ループを区別し、Sell 判定も出せるよう拡張する必要がある。

## What Changes

- **BREAKING**: `kabu watchlist` CLI サブコマンド（add/remove/list）を廃止。watchlist テーブルは DB に残るが、discover の内部データとして使用
- `kabu discover` コマンドを新設。Gemini CLI で投資 Spec に基づき有望銘柄を自動発掘し、watchlist を自動管理する
- `kabu discover --list` で現在の watchlist（discover が追跡中の銘柄）を確認可能
- eval の判定を Buy/Hold/Avoid の3択から Buy/Hold/Sell/Avoid の4択に拡張
- eval に Hunting（新規候補）と Farming（保有管理）の2ループを導入。保有中銘柄は portfolio_positions から自動取得し、Sell 判定を出せるようにする
- eval の出力 JSON を拡張（status, catalyst_check, risk_assessment, spec_compliance, execution_instruction）
- execute が Sell decision を処理できるよう拡張

## Capabilities

### New Capabilities
- `stock-discovery`: Gemini CLI を使った投資 Spec ベースの自動銘柄発掘と watchlist 自動管理

### Modified Capabilities
- `watchlist`: CLI コマンドを廃止し、discover の内部データソースに変更
- `investment-evaluation`: Hunting/Farming の2ループ導入、4択判定（Buy/Hold/Sell/Avoid）、拡張 JSON 出力
- `trade-execution`: Sell decision の処理を追加

## Impact

- **CLI**: `kabu watchlist` サブコマンド削除、`kabu discover` 追加
- **Code**: `src/cmd/watchlist.rs` 削除、`src/cmd/discover.rs` 新設、`src/cmd/eval.rs` 大幅改修、`src/cmd/execute.rs` Sell 対応
- **DB**: watchlist テーブルは変更なし（discover が内部利用）。evaluations テーブルに status カラム追加の可能性
- **LLM prompt**: eval プロンプトに Hunting/Farming 区分とポートフォリオ状況を含める
- **Dependencies**: 新規依存なし（Gemini CLI は既存の llm/cli_gemini.rs を活用）
