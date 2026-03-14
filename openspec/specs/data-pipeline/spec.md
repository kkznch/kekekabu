## Purpose

J-Quants V2 API からの価格データ取得、テクニカル指標算出、トレーディングシグナル検出を一括で行う scan コマンドの仕様。

## Requirements

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

### Requirement: テクニカル指標を算出
システムは SHALL 取得した価格データからテクニカル指標（SMA, EMA, RSI, MACD, ボリンジャーバンド, ATR, 出来高MA）を計算する。

#### Scenario: 全指標の算出
- **WHEN** 十分な価格データ（75データポイント以上）が存在する場合
- **THEN** SMA(5/25/75), EMA(12/26), RSI(14), MACD(12,26,9), BB(20,2), ATR(14), 出来高MA(20) を算出し、scan 出力に含める

#### Scenario: データ不足
- **WHEN** 指標の計算に必要なデータポイント数に満たない場合
- **THEN** その指標について空の結果を返す（エラーにはしない）

### Requirement: トレーディングシグナルを検出
システムは SHALL 算出した指標からトレーディングシグナルを検出する。

#### Scenario: ゴールデンクロスの検出
- **WHEN** SMA(5) が SMA(25) を上抜けした場合
- **THEN** signals 配列に "golden_cross_5_25" を含める

#### Scenario: デッドクロスの検出
- **WHEN** SMA(5) が SMA(25) を下抜けした場合
- **THEN** signals 配列に "dead_cross_5_25" を含める

#### Scenario: 出来高急増の検出
- **WHEN** 最新の出来高が出来高MA(20) の2倍を超えた場合
- **THEN** signals 配列に "volume_spike" を含める

### Requirement: 価格データのデータベース保存
システムは SHALL 取得した価格データを冪等書き込みで SQLite データベースに永続化する。

#### Scenario: 冪等な価格保存
- **WHEN** 同一 ticker・同一日付の価格データが2回保存された場合
- **THEN** 既存レコードを最新データで更新する（ON CONFLICT DO UPDATE）
