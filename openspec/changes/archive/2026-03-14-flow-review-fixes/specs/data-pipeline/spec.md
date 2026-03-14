## MODIFIED Requirements

### Requirement: J-Quants V2 API から価格データを取得
システムは SHALL `kabu scan` 実行時に、ウォッチリスト全銘柄の日足 OHLCV データを J-Quants V2 API から取得する。

#### Scenario: ウォッチリスト銘柄の scan 成功
- **WHEN** ウォッチリストに銘柄がある状態で `kabu scan --days 60` を実行した場合
- **THEN** 各銘柄の価格データを J-Quants V2 API から取得し、DB に保存し、scan 結果を JSON で stdout に出力する

#### Scenario: ウォッチリストが空
- **WHEN** ウォッチリストが空の状態で `kabu scan` を実行した場合
- **THEN** 空の JSON 配列 `[]` を stdout に出力する

#### Scenario: API 呼び出しのレートリミット
- **WHEN** 複数銘柄のデータを取得する場合
- **THEN** J-Quants API の連続呼び出しの間に最低1秒の待機時間を設ける

#### Scenario: API タイムアウトとリトライ
- **WHEN** J-Quants API への HTTP リクエストがタイムアウトまたは 429/5xx エラーを返した場合
- **THEN** 30秒のタイムアウトを設定し、指数バックオフで最大3回リトライする

### Requirement: 価格データのデータベース保存
システムは SHALL 取得した価格データを冪等書き込みで SQLite データベースに永続化する。

#### Scenario: 冪等な価格保存
- **WHEN** 同一 ticker・同一日付の価格データが2回保存された場合
- **THEN** 既存レコードを最新データで更新する（ON CONFLICT DO UPDATE）
