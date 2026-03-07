# kekekabu (kabu)

日本株投資のための CLI ツール。LLM を活用した銘柄評価パイプラインを提供します。

## パイプライン

```
scan → fetch → eval → execute → report
```

| コマンド | 概要 |
|---------|------|
| `scan` | J-Quants API から価格データを取得し、テクニカル指標（RSI, MACD, BB, SMA 等）を算出 |
| `fetch` | LLM で最新ニュース・開示・センチメント等の情報を収集 |
| `eval` | LLM（Claude / Gemini）で投資判断（Buy / Hold / Avoid）を生成 |
| `execute` | サーキットブレーカー確認後、売買シグナルを出力 |
| `report` | 評価結果を Markdown レポートとして出力 |

### 依存関係マトリクス

| コマンド | DB | LLM | 外部 API |
|---------|:--:|:---:|:--------:|
| `scan` | W | - | J-Quants |
| `fetch` | R/W | ✓ | - |
| `eval` | R/W | ✓ | - |
| `execute` | R | - | - |
| `report` | R | - | - |
| `watchlist` | R/W | - | - |
| `portfolio` | R/W | - | - |
| `history` | R | - | - |
| `init` | - | - | - |

> R = 読み取り、W = 書き込み、R/W = 両方

## セットアップ

```sh
# ツールのインストール
aqua install

# ビルド
just build

# 設定ファイルの初期化
cargo run -- init
# → ~/.config/kabu/config.toml と specs/template.yaml が生成される
```

`~/.config/kabu/config.toml` を編集して設定してください。

### `[api]` — API キー

| キー | 説明 | 必須 |
|------|------|------|
| `jquants_api_key` | [J-Quants API](https://jpx.gitbook.io/j-quants-ja) のキー。`scan` で価格データ取得に使用 | `scan` 使用時 |
| `anthropic_api_key` | Anthropic API キー。`llm.eval = "api-anthropic"` の場合に使用 | `api-anthropic` 使用時 |
| `gemini_api_key` | Google Gemini API キー。`llm.fetch = "api-gemini"` の場合に使用 | `api-gemini` 使用時 |

環境変数 `JQUANTS_API_KEY`, `ANTHROPIC_API_KEY`, `GEMINI_API_KEY` でも設定可能です（config より優先）。

### `[llm]` — LLM バックエンド

| キー | デフォルト | 説明 |
|------|-----------|------|
| `fetch` | `cli-gemini` | `fetch` コマンドで使う LLM。`cli-gemini` / `cli-claude` / `api-gemini` / `api-anthropic` |
| `eval` | `cli-claude` | `eval` コマンドで使う LLM。同上 |
| `fetch_model` | (なし) | `fetch` で使うモデル名の上書き |
| `eval_model` | (なし) | `eval` で使うモデル名の上書き |

`cli-gemini` / `cli-claude` はそれぞれ `gemini` / `claude` CLI がインストールされている必要があります。

### `[spec]` — 投資戦略

| キー | デフォルト | 説明 |
|------|-----------|------|
| `path` | `specs/template.yaml` | 投資戦略 YAML ファイルのパス（config ディレクトリからの相対パスまたは絶対パス） |

`kabu init` で生成される `template.yaml` をコピーして独自の戦略ファイルを作成し、ここで指定します。

### `[output]` — 出力設定

| キー | デフォルト | 説明 |
|------|-----------|------|
| `default_format` | `json` | デフォルトの出力形式。`json` または `human` |

### 設定例

```toml
[api]
jquants_api_key = "YOUR_JQUANTS_API_KEY"

[llm]
fetch = "cli-gemini"
eval = "cli-claude"

[spec]
path = "specs/my-strategy.yaml"

[output]
default_format = "json"
```

## 使い方

```sh
# 日次パイプライン
kabu scan --days 60
kabu fetch
kabu eval
kabu execute --dry-run
kabu report -o report.md

# ウォッチリスト管理
kabu watchlist add 7203
kabu watchlist list
kabu watchlist remove 7203

# ポートフォリオ管理
kabu portfolio buy 7203 --quantity 100 --price 2000
kabu portfolio sell 7203 --quantity 50 --price 2200
kabu portfolio positions
kabu portfolio summary
kabu portfolio trades

# 評価履歴
kabu history --limit 20
```

出力はデフォルトで JSON（stdout）。`--format human` で人間向け表示に切り替え可能。
ログは stderr に出力されるため、パイプラインでの利用に適しています。

## 自動化（cron / launchd）

```sh
# 朝: データ収集 → 評価
kabu scan --days 60 && kabu fetch && kabu eval

# 市場オープン: 実行
kabu execute

# 夕方: レポート生成
kabu report -o ~/reports/$(date +%Y-%m-%d).md
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
- **LLM**: Anthropic API / Gemini API / Claude CLI / Gemini CLI
- **テクニカル分析**: rust_ti（RSI, MACD, BB, SMA, EMA, ATR）
- **金額精度**: rust_decimal（TEXT 保存）

## 安全機構

- **サーキットブレーカー**: 個別銘柄 >30% 変動、またはウォッチリストの >50% が >5% 下落した場合に execute をブロック
- **ドライラン**: `execute --dry-run` がデフォルト
- **投資 Spec**: YAML で戦略パラメータを外部管理、SHA256 ハッシュで評価時の Spec を追跡
