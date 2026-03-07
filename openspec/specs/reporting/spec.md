## Purpose

評価結果からの Markdown レポート生成。Buy/Hold/Avoid カテゴリ別に集約し、TA 詳細を含めて出力する。

## Requirements

### Requirement: 評価結果から Markdown レポートを生成
システムは SHALL evaluations を Buy/Hold/Avoid カテゴリ別にグルーピングした Markdown レポートを生成する。

#### Scenario: stdout へのレポート出力
- **WHEN** ユーザーが `kabu report` を実行した場合
- **THEN** 当日の評価結果を含む Markdown レポートを stdout に出力する

#### Scenario: ファイルへのレポート出力
- **WHEN** ユーザーが `kabu report -o report.md` を実行した場合
- **THEN** 指定されたファイルパスに Markdown レポートを書き出す

#### Scenario: 特定日付のレポート
- **WHEN** ユーザーが `kabu report --date 2026-03-07` を実行した場合
- **THEN** 指定された日付の評価結果を使ってレポートを生成する

### Requirement: レポートに TA 詳細を含める
システムは SHALL 各銘柄のテクニカル分析詳細（シグナル、指標値）をレポートに含める。

#### Scenario: シグナルを持つ銘柄
- **WHEN** ある銘柄にトレーディングシグナル（ゴールデンクロス、出来高急増等）が検出されている場合
- **THEN** その銘柄のセクションにシグナルを含める
