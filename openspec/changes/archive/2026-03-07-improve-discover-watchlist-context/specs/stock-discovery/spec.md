## MODIFIED Requirements

### Requirement: Gemini CLI による自動銘柄発掘
システムは SHALL `kabu discover` 実行時に、投資 Spec の条件と現在のウォッチリストに基づいて Gemini CLI で有望な日本株銘柄を発掘し、watchlist テーブルを自動更新する。Budget Context が利用可能な場合はプロンプトに含め、資金状況を考慮した銘柄選定を行う。プロンプトには現在のウォッチリスト銘柄を含め、LLM に keep/add/remove のアクション別判断を求める。

#### Scenario: 銘柄発掘の成功
- **WHEN** 有効な投資 Spec が設定された状態で `kabu discover` を実行した場合
- **THEN** Gemini CLI に投資 Spec と現在のウォッチリストを含むプロンプトを送信し、keep/add/remove のアクション別銘柄リストを JSON で受け取り、watchlist テーブルを更新する

#### Scenario: ウォッチリストが空の状態での発掘
- **WHEN** watchlist が空の状態で `kabu discover` を実行した場合
- **THEN** 「現在の追跡銘柄なし」としてプロンプトを構築し、add のみの結果を返す

#### Scenario: Budget Context 付きの銘柄発掘
- **WHEN** Spec に `[budget]` セクションが定義された状態で `kabu discover` を実行した場合
- **THEN** プロンプトに Budget Context セクションを含め、残り投資可能額と100株単元での購入可能性を考慮した銘柄を選定させる

#### Scenario: Budget Context なしの銘柄発掘
- **WHEN** Spec に `[budget]` セクションが存在しない状態で `kabu discover` を実行した場合
- **THEN** 従来どおり Budget Context なしでプロンプトを送信する

#### Scenario: LLM 判断に基づく差分管理
- **WHEN** discover が keep/add/remove のアクション別レスポンスを返した場合
- **THEN** add の銘柄を watchlist に追加し、remove の銘柄を watchlist から削除する。ただし portfolio_positions に保有中の銘柄は remove でも削除しない

#### Scenario: 不正な ticker の無視
- **WHEN** Gemini CLI が不正な形式の ticker（数字4桁でない）を返した場合
- **THEN** その ticker をスキップし、警告をログ出力する

### Requirement: discover の出力を JSON でパース
システムは SHALL Gemini CLI の応答を keep/add/remove のアクション別 ticker リストとしてパースする。

#### Scenario: 正常な JSON パース
- **WHEN** Gemini CLI が keep/add/remove を含む有効な JSON を返した場合
- **THEN** 各アクションの ticker を抽出し、watchlist の更新に使用する

#### Scenario: パース失敗
- **WHEN** Gemini CLI の応答が有効な JSON でない場合
- **THEN** エラーを返し、watchlist を変更しない
