## ADDED Requirements

### Requirement: 全上場銘柄マスターを一括取得
システムは SHALL `kabu scan --refresh-master` 実行時に J-Quants V2 `equities/master` API を呼び出し、全上場銘柄の情報（コード、社名、セクター）を一括取得して stocks テーブルに UPSERT する。

#### Scenario: 一括取得の成功
- **WHEN** `kabu scan --refresh-master --days 60` を実行した場合
- **THEN** 全上場銘柄（約4000件）を1回の API 呼び出しで取得し、stocks テーブルに UPSERT した後、通常の scan 処理を続行する

#### Scenario: 既存銘柄の更新
- **WHEN** stocks テーブルに既に存在する銘柄の社名またはセクターが変更されていた場合
- **THEN** UPSERT により最新の情報に更新する

#### Scenario: API エラー時
- **WHEN** equities/master API が 429 以外のエラーを返した場合
- **THEN** エラーメッセージを表示して scan を中断する

### Requirement: stocks テーブル空時のエラー案内
システムは SHALL `--refresh-master` なしで scan を実行した際に stocks テーブルが空の場合、エラーメッセージで `--refresh-master` の使用を案内する。

#### Scenario: 初回実行でマスター未取得
- **WHEN** stocks テーブルが空の状態で `kabu scan --days 60`（`--refresh-master` なし）を実行した場合
- **THEN** `stocks テーブルが空です。先に kabu scan --refresh-master を実行してください` というエラーを表示する

#### Scenario: マスター取得済みの通常実行
- **WHEN** stocks テーブルにデータがある状態で `kabu scan --days 60`（`--refresh-master` なし）を実行した場合
- **THEN** stocks テーブルの既存データを参照して通常の scan 処理を実行する
