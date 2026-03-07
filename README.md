# keketrade (kktd)

日本株投資のための CLI ツール。LLM を活用した銘柄評価パイプラインを提供します。

## パイプライン

```
scan → fetch → eval → execute → report
```

| コマンド | 概要 |
|---------|------|
| `scan` | J-Quants API から価格データを取得し、テクニカル指標（RSI, MACD, BB, SMA 等）を算出 |
| `fetch` | Gemini CLI で最新ニュース・開示・センチメント等の情報を収集 |
| `eval` | LLM（Claude / Gemini）で投資判断（Buy / Hold / Avoid）を生成 |
| `execute` | サーキットブレーカー確認後、売買シグナルを出力 |
| `report` | 評価結果を Markdown レポートとして出力 |

## セットアップ

```sh
# ツールのインストール
aqua install

# ビルド
just build

# 設定ファイルの初期化
cargo run -- init
# → ~/.config/kktd/config.toml と specs/template.yaml が生成される
```

`~/.config/kktd/config.toml` を編集して API キーを設定してください。

```toml
[api]
jquants_api_key = "YOUR_JQUANTS_API_KEY"
anthropic_api_key = "YOUR_ANTHROPIC_API_KEY"

[llm]
fetch = "cli-gemini"    # fetch コマンドの LLM バックエンド
eval = "cli-claude"     # eval コマンドの LLM バックエンド

[spec]
path = "specs/template.yaml"   # 投資戦略ファイルのパス
```

環境変数でも設定可能です: `JQUANTS_API_KEY`, `ANTHROPIC_API_KEY`

## 使い方

```sh
# 日次パイプライン
kktd scan --days 60
kktd fetch
kktd eval
kktd execute --dry-run
kktd report -o report.md

# ウォッチリスト管理
kktd watchlist add 7203
kktd watchlist list
kktd watchlist remove 7203

# ポートフォリオ管理
kktd portfolio buy 7203 --quantity 100 --price 2000
kktd portfolio sell 7203 --quantity 50 --price 2200
kktd portfolio positions
kktd portfolio summary
kktd portfolio trades

# 評価履歴
kktd history --limit 20
```

出力はデフォルトで JSON（stdout）。`--format human` で人間向け表示に切り替え可能。
ログは stderr に出力されるため、パイプラインでの利用に適しています。

## 自動化（cron / launchd）

```sh
# 朝: データ収集 → 評価
kktd scan --days 60 && kktd fetch && kktd eval

# 市場オープン: 実行
kktd execute

# 夕方: レポート生成
kktd report -o ~/reports/$(date +%Y-%m-%d).md
```

## 開発

```sh
aqua install        # ツールインストール（just 等）
just build          # ビルド
just test           # テスト実行
just lint           # Clippy
just ci             # fmt-check + lint + test
just --list         # タスク一覧
```

## 技術スタック

- **言語**: Rust 2024 edition
- **DB**: SQLite（tokio-rusqlite, bundled）
- **API**: J-Quants V2
- **LLM**: Anthropic API / Claude CLI / Gemini CLI
- **テクニカル分析**: rust_ti（RSI, MACD, BB, SMA, EMA, ATR）
- **金額精度**: rust_decimal（TEXT 保存）

## 安全機構

- **サーキットブレーカー**: 個別銘柄 >30% 変動、またはウォッチリストの >50% が >5% 下落した場合に execute をブロック
- **ドライラン**: `execute --dry-run` がデフォルト
- **投資 Spec**: YAML で戦略パラメータを外部管理、SHA256 ハッシュで評価時の Spec を追跡
