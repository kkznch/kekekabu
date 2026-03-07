## Purpose

LLM による投資判断生成（Buy/Hold/Avoid）。TA 指標・収集情報・投資 Spec を統合したプロンプトで評価を行う。

## Requirements

### Requirement: LLM による投資判断の生成
システムは SHALL TA 指標、fetch 結果、投資 Spec を統合した包括的なプロンプトを構築し、eval 用 LLM バックエンドに送信して投資判断を生成する。

#### Scenario: 評価成功
- **WHEN** scan と fetch のデータがあるウォッチリスト銘柄に対して `kabu eval` を実行した場合
- **THEN** 各銘柄について Buy/Hold/Avoid の判断、スコア（0-100）、根拠を生成する

#### Scenario: 特定銘柄の評価
- **WHEN** `kabu eval 7203` を実行した場合
- **THEN** 指定された銘柄のみを評価する

### Requirement: 評価応答のフォーマット
システムは SHALL LLM 応答を `decision`（Buy/Hold/Avoid）、`score`（0-100）、`rationale`（summary, technical, risks）を含む JSON としてパースする。

#### Scenario: 有効な eval 応答
- **WHEN** LLM が適切なフォーマットの JSON 応答を返した場合
- **THEN** decision, score, rationale フィールドを抽出する

#### Scenario: Markdown でラップされた eval 応答
- **WHEN** LLM が markdown コードブロックでラップされた JSON を返した場合
- **THEN** パース前にコードブロックから JSON を抽出する

### Requirement: eval プロンプトに投資 Spec を含める
システムは SHALL eval プロンプトに投資 Spec（ユニバースフィルタ、スコアリング要因、執行パラメータ）を含める。

#### Scenario: Spec のプロンプト埋め込み
- **WHEN** Spec ファイルが設定された状態で eval コマンドを実行した場合
- **THEN** Spec YAML を読み込み、プロンプトセクションに変換して LLM プロンプトに含める

### Requirement: 評価結果を Spec ハッシュ付きで永続化
システムは SHALL 評価結果を、使用した Spec の SHA256 ハッシュ付きでデータベースに保存する。

#### Scenario: spec_hash 付き評価保存
- **WHEN** 評価が完了した場合
- **THEN** ticker, name, decision, score, rationale, spec_hash, evaluated_at を `evaluations` テーブルに保存する
