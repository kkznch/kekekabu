## 1. watchlist CLI 廃止

- [x] 1.1 main.rs から WatchlistCommand enum と Watchlist(WatchlistCommand) を削除
- [x] 1.2 src/cmd/watchlist.rs を削除し、cmd/mod.rs から `pub mod watchlist` を削除
- [x] 1.3 CLAUDE.md、README.md から watchlist CLI の記述を削除

## 2. discover コマンド実装

- [x] 2.1 src/cmd/discover.rs を作成し、discover のプロンプト構築・LLM 呼び出し・JSON パースを実装
- [x] 2.2 discover の watchlist 差分管理を実装（新規追加、リスト外削除、保有中銘柄は維持）
- [x] 2.3 discover --list オプションを実装（db::watchlist_list を呼ぶだけ）
- [x] 2.4 main.rs に Discover コマンドを追加し、cmd/mod.rs に `pub mod discover` を追加
- [x] 2.5 discover のユニットテスト（JSON パース、ticker バリデーション）を追加

## 3. eval 拡張（Hunting/Farming + 4択判定）

- [x] 3.1 EvalResponse を拡張（status, analysis, execution_instruction フィールド追加、rationale を analysis に置換）
- [x] 3.2 eval の対象銘柄取得ロジックを変更（watchlist + portfolio_positions の両方から取得、status を付与）
- [x] 3.3 eval プロンプトを Hunting/Farming 対応に書き換え（Buy/Hold/Sell/Avoid 4択、保有情報の埋め込み）
- [x] 3.4 parse_eval_response を新 JSON フォーマットに対応させる
- [x] 3.5 eval のユニットテストを新フォーマットに更新

## 4. execute の Sell 対応

- [x] 4.1 execute に Sell decision の処理を追加（portfolio_positions を確認し、保有があれば売りシグナル生成）
- [x] 4.2 execute のテストがあれば Sell ケースを追加

## 5. ドキュメント・OpenSpec 更新

- [x] 5.1 CLAUDE.md のパイプライン記述を discover → scan → fetch → eval → execute → report に更新
- [x] 5.2 README.md のパイプライン、コマンド一覧、依存関係マトリクス、使い方を更新
- [x] 5.3 OpenSpec の watchlist spec を REMOVED 状態に反映（archive 時に自動処理）

## 6. テスト・検証

- [x] 6.1 cargo test で全テスト通過を確認
- [x] 6.2 cargo clippy で警告がないことを確認
