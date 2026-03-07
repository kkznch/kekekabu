## Context

kekekabu は日本株投資の CLI ツールで、5段階パイプライン（scan → fetch → eval → execute → report）で動作する。現在は銘柄を手動で `kabu watchlist add` する必要があり、eval は新規候補のみを Buy/Hold/Avoid の3択で評価する。保有中銘柄の売却判断は eval の対象外で、execute も Sell decision を処理できない。

Gemini からの提案（.keke/gemini.md）により、自律パイプラインへの進化が求められている。

## Goals / Non-Goals

**Goals:**
- discover コマンドで Gemini CLI が投資 Spec ベースで銘柄を自動発掘する
- eval が Hunting（新規候補）と Farming（保有管理）の2ループを区別し、4択判定を出す
- execute が Sell decision を処理できる
- watchlist CLI を廃止し、ユーザーが触る必要のない内部データに変更する

**Non-Goals:**
- Tachibana API の実売買接続（引き続きスタブ）
- discover で J-Quants API を直接呼ぶ（discover は LLM ベースの銘柄発掘のみ、scan がデータ取得を担う）
- ポートフォリオ全体のリバランス最適化（eval は個別銘柄の判断に集中）

## Decisions

### Decision 1: discover は Gemini CLI を直接使う

discover コマンドは既存の `llm::create_backend` + `cli_gemini` を使って Gemini CLI にプロンプトを送り、銘柄リスト JSON を受け取る。

**Rationale:** 新しい依存を追加せず、既存の LLM インフラを再利用できる。fetch コマンドと同じアプローチ。

**Alternative:** J-Quants のスクリーニング API を使う → 投資 Spec の「質的な判断」（成長ストーリー、市場トレンド）を反映できないため却下。

### Decision 2: eval の Hunting/Farming 区別は呼び出し側で行う

eval コマンド内で watchlist と portfolio_positions を両方クエリし、各銘柄に `status: NewTarget | ExistingHolding` を付与してからプロンプトに渡す。

**Rationale:** eval の中で完結するため、パイプラインの他コマンドに影響しない。

### Decision 3: eval の出力 JSON を拡張するが後方互換を維持

既存の `decision`, `score`, `rationale` フィールドはそのまま残し、新フィールド（`status`, `analysis`, `execution_instruction`）を追加する。`rationale` の中身を `analysis` にリネームするのは破壊的変更になるため、`rationale` を残しつつ `analysis` も受け付けるようにする。

**Rationale:** report コマンドや既存の evaluations テーブルへの影響を最小化。

**Decision:** `rationale` を廃止し `analysis` に統一する。既にユーザーがいないツールなので後方互換は不要。

### Decision 4: execute の Sell 処理

eval が `decision: "Sell"` を出した場合、execute は portfolio_positions を確認し、保有していれば売りシグナルを生成する。保有していなければスキップ。

### Decision 5: discover --list は DB の watchlist テーブルを直接クエリ

`discover --list` は新しい discover ロジックを呼ばず、単に `db::watchlist_list()` を呼ぶ。既存の list 機能のシンプルな移植。

### Decision 6: watchlist の DB 関数は残す

`db::watchlist_add`, `db::watchlist_remove`, `db::watchlist_list` はそのまま残す。discover が内部的に使う。CLI コマンド（`src/cmd/watchlist.rs`）と main.rs の `WatchlistCommand` のみ削除。

## Risks / Trade-offs

- **[Risk] discover の LLM 出力が不安定** → JSON パースのリトライとバリデーション（ticker 形式チェック）でカバー。不正な ticker は無視。
- **[Risk] eval プロンプトの肥大化** → Hunting と Farming で別プロンプトにする選択肢もあるが、1つのプロンプトに統合してコンテキストを共有する方がLLMの判断精度が上がると判断。
- **[Trade-off] watchlist CLI 削除は BREAKING** → ユーザー（=開発者本人のみ）なので問題なし。
