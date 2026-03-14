## MODIFIED Requirements

### Requirement: Gemini CLI による自動銘柄発掘
システムは SHALL `kabu discover` 実行時に、投資 Spec の条件と現在のウォッチリストに基づいて Gemini CLI で有望な日本株銘柄を発掘し、watchlist テーブルを自動更新する。Budget Context が利用可能な場合はプロンプトに含め、資金状況を考慮した銘柄選定を行う。プロンプトには現在のウォッチリスト銘柄を含め、LLM に keep/add/remove のアクション別判断を求める。

#### Scenario: 銘柄発掘の成功
- **WHEN** 有効な投資 Spec が設定された状態で `kabu discover` を実行した場合
- **THEN** Gemini CLI に投資 Spec と現在のウォッチリストを含むプロンプトを送信し、keep/add/remove のアクション別銘柄リストを JSON で受け取り、watchlist テーブルを更新する

#### Scenario: 不正な ticker の無視
- **WHEN** Gemini CLI が不正な形式の ticker（数字4〜5桁でない）を返した場合
- **THEN** その ticker をスキップし、警告をログ出力する

#### Scenario: add/keep 間の重複排除
- **WHEN** 同一 ticker が add リストと keep リストの両方に含まれている場合
- **THEN** 重複を排除し、1銘柄として処理する

#### Scenario: LLM 判断に基づく差分管理
- **WHEN** discover が keep/add/remove のアクション別レスポンスを返した場合
- **THEN** add の銘柄を watchlist に追加し、remove の銘柄を watchlist から削除する。ただし portfolio_positions に保有中の銘柄は remove でも削除しない
