## Purpose

SQLite による永続化層。7 テーブル構成で冪等書き込みを保証し、rust_decimal による金額精度を維持する。全コマンドのデータ基盤。

## Requirements

### Requirement: SQLite を使った7テーブル構成のデータベース
システムは SHALL SQLite（tokio-rusqlite, bundled）を使用し、stocks, prices, watchlist, evaluations, fetch_results, portfolio_positions, trades の7テーブルを管理する。

#### Scenario: データベース初期化
- **WHEN** アプリケーション起動時（init 以外の任意のコマンド）
- **THEN** ~/.config/kabu/kekekabu.db にデータベースファイルを作成し、7テーブルすべてが存在することを保証する

### Requirement: stocks テーブル
システムは SHALL 銘柄マスタデータ（ticker, name, sector）を ticker をユニークキーとして保存する。

#### Scenario: 銘柄の upsert
- **WHEN** 同一 ticker で name/sector が更新された銘柄データが保存された場合
- **THEN** 既存レコードを更新する（ON CONFLICT UPDATE）

### Requirement: prices テーブル
システムは SHALL 日足 OHLCV データを (ticker, date) をユニークキーとして保存する。

#### Scenario: 冪等な価格データ挿入
- **WHEN** 同一 ticker・同一日付の価格データが再度挿入された場合
- **THEN** 重複を無視する（INSERT OR IGNORE）

### Requirement: 金額を TEXT 型で保存
システムは SHALL すべての金額を SQLite 上で TEXT 型として保存し、rust_decimal::Decimal で精度を保証する。

#### Scenario: Decimal の精度保持
- **WHEN** 2345.50 という価格を保存して読み戻した場合
- **THEN** 浮動小数点の丸め誤差なく正確な値が復元される

### Requirement: evaluations テーブルに spec_hash を記録
システムは SHALL 評価結果を、使用した投資 Spec の SHA256 ハッシュとともに保存する。

#### Scenario: Spec 追跡付き評価保存
- **WHEN** 評価が保存される場合
- **THEN** 評価に使用した Spec バージョンに紐づく spec_hash フィールドが含まれる

### Requirement: fetch_results テーブル
システムは SHALL LLM が収集した情報（category, content, source）を ticker ごとに保存する。

#### Scenario: 1銘柄に対する複数カテゴリのデータ
- **WHEN** fetch がある銘柄のニュース、開示、センチメントを収集した場合
- **THEN** それぞれを適切な category で別行として保存する

### Requirement: ポートフォリオ関連テーブル
システムは SHALL portfolio_positions（保有ポジション・加重平均コスト）と trades（売買履歴・P&L）テーブルを使用する。

#### Scenario: ポジションのライフサイクル
- **WHEN** 買い → 一部売り → 全売りの一連の操作が行われた場合
- **THEN** portfolio_positions が quantity/avg_cost を追跡し、trades が各取引を P&L 付きで記録する
