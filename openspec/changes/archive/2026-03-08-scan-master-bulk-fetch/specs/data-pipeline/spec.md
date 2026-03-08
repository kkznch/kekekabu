## MODIFIED Requirements

### Requirement: J-Quants V2 API から価格データを取得
システムは SHALL `kabu scan` 実行時に、ウォッチリスト全銘柄の日足 OHLCV データを J-Quants V2 API から取得する。銘柄情報（社名・セクター）は stocks テーブルから参照し、個別の stock_info API 呼び出しは行わない。

#### Scenario: ウォッチリスト銘柄の scan 成功
- **WHEN** ウォッチリストに銘柄がある状態で `kabu scan --days 60` を実行した場合
- **THEN** 各銘柄の価格データを J-Quants V2 API から取得し、DB に保存し、scan 結果を JSON で stdout に出力する

#### Scenario: ウォッチリストが空
- **WHEN** ウォッチリストが空の状態で `kabu scan` を実行した場合
- **THEN** エラーメッセージを表示する

#### Scenario: API 呼び出しのレートリミット
- **WHEN** 複数銘柄のデータを取得する場合
- **THEN** J-Quants API の連続呼び出しの間に最低0.3秒の待機時間を設ける

#### Scenario: watchlist の銘柄が stocks テーブルに未登録
- **WHEN** watchlist に含まれる銘柄が stocks テーブルに存在しない場合
- **THEN** 当該銘柄をスキップし、警告ログを出力する
