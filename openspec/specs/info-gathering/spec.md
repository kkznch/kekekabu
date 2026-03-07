## Purpose

LLM を使った銘柄情報収集（ニュース、適時開示、センチメント、競合分析）を行い、eval の入力データを構築する。

## Requirements

### Requirement: LLM による銘柄情報の収集
システムは SHALL 設定された fetch 用 LLM バックエンドを使用し、ウォッチリスト銘柄の最新情報（ニュース、開示、センチメント、競合分析）を収集する。

#### Scenario: ウォッチリスト全銘柄の fetch 成功
- **WHEN** ウォッチリストに銘柄がある状態で `kabu fetch` を実行した場合
- **THEN** 各銘柄について構造化プロンプトを fetch 用 LLM バックエンドに送信し、結果を `fetch_results` テーブルに保存する

#### Scenario: 特定銘柄の fetch
- **WHEN** `kabu fetch 7203 6758` を実行した場合
- **THEN** 指定された銘柄（ウォッチリストに含まれるもの）のみ情報を収集する

### Requirement: 収集結果の構造化
システムは SHALL LLM の応答をニュース、開示、センチメント、競合情報を含む構造化 JSON としてパースする。

#### Scenario: 有効な LLM 応答
- **WHEN** LLM が期待されるフィールドを含む JSON 応答を返した場合
- **THEN** パースして個別アイテムを category と content 付きで `fetch_results` テーブルに保存する

#### Scenario: Markdown でラップされた JSON 応答
- **WHEN** LLM が markdown コードブロック（```json ... ```）でラップされた JSON を返した場合
- **THEN** コードブロックから JSON を抽出して正しくパースする

### Requirement: 収集結果の永続化
システムは SHALL fetch 結果を eval コマンドで使用するためにデータベースに保存する。

#### Scenario: タイムスタンプ付き保存
- **WHEN** fetch 結果が保存される場合
- **THEN** 各結果に ticker, category, content, source, fetched_at タイムスタンプを含める
