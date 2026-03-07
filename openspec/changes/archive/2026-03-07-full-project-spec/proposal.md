## Why

kekekabu (kabu) は日本株投資のための CLI ツールで、LLM を活用した銘柄評価パイプライン（scan → fetch → eval → execute → report）を提供する。旧リポジトリ kekestock から Rust 2024 edition で全面移植し、5コマンドパイプライン + ポートフォリオ管理 + 安全機構を備えた投資支援ツールとして完成させた。全仕様を OpenSpec でドキュメント化する。

## What Changes

- J-Quants V2 API による価格データ取得とテクニカル指標算出（scan）
- LLM（Gemini CLI / Claude CLI / Anthropic API / Gemini API）による情報収集（fetch）と投資判断（eval）
- サーキットブレーカー付き売買シグナル出力（execute）
- Markdown レポート生成（report）
- ウォッチリスト CRUD（watchlist add/remove/list）
- ポートフォリオ管理（buy/sell/positions/summary/trades）with 加重平均コスト・P&L 計算
- 設定ファイル初期化（init）with 投資 Spec テンプレート生成
- SQLite（tokio-rusqlite bundled）による永続化（7 テーブル）
- 投資 Spec TOML による戦略パラメータ外部管理（SHA256 ハッシュ追跡）
- JSON デフォルト出力 / human 表示切替
- 環境変数による設定オーバーライド

## Capabilities

### New Capabilities

- `data-pipeline`: scan コマンドによる J-Quants 価格取得 + テクニカル指標算出（RSI, MACD, BB, SMA, EMA, ATR, Volume MA）
- `llm-integration`: LLM バックエンド抽象化（trait + factory）。4 バックエンド: api-anthropic, api-gemini, cli-claude, cli-gemini
- `info-gathering`: fetch コマンドによる LLM 情報収集（ニュース、開示、センチメント、競合）
- `investment-evaluation`: eval コマンドによる LLM 投資判断（Buy/Hold/Avoid + スコア + 根拠）
- `trade-execution`: execute コマンドによるサーキットブレーカー付き売買シグナル出力
- `reporting`: report コマンドによる Markdown レポート生成（評価結果を Buy/Hold/Avoid で分類）
- `watchlist`: ウォッチリスト管理（add/remove/list）
- `portfolio`: ポートフォリオ管理（buy/sell/positions/summary/trades）with 加重平均コスト・P&L
- `config`: 設定管理（TOML config + env overrides + init コマンド + 投資 Spec テンプレート）
- `database`: SQLite 永続化層（7 テーブル、冪等書き込み）
- `safety`: サーキットブレーカー（個別銘柄 >30% 変動、市場全体 >50% 下落でブロック）

### Modified Capabilities

(なし — 新規プロジェクトのため既存の capability は存在しない)

## Impact

- **コードベース**: src/ 以下 18 ファイル（main.rs, lib.rs, config.rs, jquants.rs, indicators.rs, output.rs, portfolio.rs, circuit_breaker.rs, spec.rs, db/*, llm/*, cmd/*）
- **外部 API**: J-Quants V2 API, Anthropic Messages API, Google Gemini generateContent API
- **外部 CLI**: claude CLI, gemini CLI
- **データベース**: SQLite（~/.config/kabu/kekekabu.db）7 テーブル
- **設定**: ~/.config/kabu/config.toml, ~/.config/kabu/specs/*.toml
- **依存クレート**: clap, tokio, tokio-rusqlite, reqwest, serde, rust_decimal, rust_ti, chrono, anyhow, tracing, sha2, async-trait, which 他
