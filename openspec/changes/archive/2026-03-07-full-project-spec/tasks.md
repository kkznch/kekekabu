## 1. 設定と初期化

- [x] 1.1 AppConfig 構造体（[api], [llm], [spec], [output] セクション）
- [x] 1.2 ~/.config/kabu/config.toml からの TOML 設定読み込み
- [x] 1.3 環境変数による上書き（JQUANTS_API_KEY, ANTHROPIC_API_KEY, GEMINI_API_KEY）
- [x] 1.4 `kabu init` コマンド（config.toml + specs/template.yaml 生成）
- [x] 1.5 `--force` フラグによる設定上書き、テンプレートは常に再生成

## 2. データベース

- [x] 2.1 tokio-rusqlite（bundled）による SQLite セットアップ
- [x] 2.2 スキーマ: stocks, prices, watchlist, evaluations, fetch_results, portfolio_positions, trades
- [x] 2.3 冪等書き込み（INSERT OR IGNORE / ON CONFLICT）
- [x] 2.4 金額を TEXT 型で保存（rust_decimal::Decimal）
- [x] 2.5 全7テーブルの CRUD 操作
- [x] 2.6 DB 統合テスト（7テスト）

## 3. データパイプライン（scan）

- [x] 3.1 J-Quants V2 API クライアント（Bearer トークン認証）
- [x] 3.2 日足 OHLCV 価格データの取得
- [x] 3.3 レートリミット（API 呼び出し間に1秒待機）
- [x] 3.4 テクニカル指標: SMA(5/25/75), EMA(12/26), RSI(14), MACD, BB(20,2), ATR(14), 出来高MA(20)
- [x] 3.5 シグナル検出: ゴールデン/デッドクロス、MACD クロス、BB ブレイクアウト、出来高急増、RSI 売られすぎ/買われすぎ
- [x] 3.6 指標テスト（6テスト）

## 4. LLM 統合

- [x] 4.1 LlmBackend トレイト（send_message(prompt, max_tokens)）
- [x] 4.2 ファクトリ関数 create_backend()（ApiConfig パラメータ）
- [x] 4.3 api-anthropic バックエンド（Anthropic Messages API）
- [x] 4.4 api-gemini バックエンド（Gemini generateContent API）
- [x] 4.5 cli-claude バックエンド（claude -p）
- [x] 4.6 cli-gemini バックエンド（gemini -p）
- [x] 4.7 モデルオーバーライド対応（eval_model, fetch_model）

## 5. 情報収集（fetch）

- [x] 5.1 ニュース/開示/センチメント/競合情報の構造化プロンプト
- [x] 5.2 JSON 応答パース（markdown コードブロック抽出付き）
- [x] 5.3 fetch_results のデータベース永続化
- [x] 5.4 fetch テスト（2テスト）

## 6. 投資評価（eval）

- [x] 6.1 TA 指標 + fetch 結果 + Spec を統合した包括的プロンプト
- [x] 6.2 JSON 応答パース（decision, score, rationale）
- [x] 6.3 投資 Spec YAML ローダー（SHA256 ハッシュ付き）
- [x] 6.4 Spec の to_prompt_section()（eval プロンプトへの埋め込み）
- [x] 6.5 spec_hash 付き evaluations の永続化
- [x] 6.6 eval テスト（3テスト）+ Spec テスト（2テスト）

## 7. 売買実行（execute）

- [x] 7.1 処理前のサーキットブレーカー確認
- [x] 7.2 decision + score 閾値に基づく売買シグナル生成
- [x] 7.3 ドライランモード（デフォルト: true）
- [x] 7.4 立花証券 API 連携スタブ（API アクセス待ち）

## 8. レポート

- [x] 8.1 evaluations からの Markdown レポート生成
- [x] 8.2 Buy/Hold/Avoid カテゴリ別グルーピング
- [x] 8.3 銘柄ごとの TA 詳細の含有
- [x] 8.4 stdout またはファイルへの出力（-o フラグ）
- [x] 8.5 日付フィルター（--date フラグ）

## 9. ウォッチリスト

- [x] 9.1 watchlist add（オプション --notes 付き）
- [x] 9.2 watchlist remove
- [x] 9.3 watchlist list
- [x] 9.4 冪等な追加（INSERT OR IGNORE）

## 10. ポートフォリオ

- [x] 10.1 加重平均コスト計算付き買い
- [x] 10.2 P&L 計算付き売り
- [x] 10.3 ポジション追跡（is_active フラグ）
- [x] 10.4 ポートフォリオサマリー（position_count, total_invested, total_value, pnl）
- [x] 10.5 件数制限付き取引履歴
- [x] 10.6 ポートフォリオテスト（5テスト）

## 11. 安全機構

- [x] 11.1 個別銘柄サーキットブレーカー（日次30%超変動）
- [x] 11.2 市場全体サーキットブレーカー（ウォッチリストの50%超が5%超下落）
- [x] 11.3 サーキットブレーカー理由の報告

## 12. 出力と CLI

- [x] 12.1 OutputFormat 列挙型（Json/Human）と clap derive
- [x] 12.2 全出力型に対する HumanDisplay トレイト実装
- [x] 12.3 JSON 出力は stdout、ログは stderr（tracing）
- [x] 12.4 グローバル --format フラグ

## 13. ツール設定

- [x] 13.1 aqua.yaml に casey/just
- [x] 13.2 justfile に build/test/lint/ci タスク
- [x] 13.3 README.md（フルドキュメント）
- [x] 13.4 CLAUDE.md（アーキテクチャ概要）
