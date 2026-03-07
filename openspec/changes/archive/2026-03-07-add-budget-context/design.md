## Context

discover/eval の LLM プロンプトに資金情報が含まれておらず、予算を無視した推薦が行われる。証券API（立花証券）は未接続で、リアルタイム残高取得は不可能。ただし DB の trades テーブルに売買履歴があるため、内部追跡は可能。

## Goals / Non-Goals

**Goals:**
- Spec TOML に `[budget]` セクションを追加し、初期投資資金を定義可能にする
- DB の trades テーブルから投資済み額・回収額を集計し、残り投資可能額を算出する
- discover / eval プロンプトに Budget Context を注入する
- budget セクションが未定義の場合はスキップする（後方互換性）

**Non-Goals:**
- 証券APIからのリアルタイム残高取得（将来対応）
- budget 超過時の自動ブロック（LLM への情報提供のみ）
- config.toml への budget 追加（Spec の責務）

## Decisions

### Decision 1: budget 情報は Spec TOML に配置する
config.toml ではなく Spec TOML の `[budget]` セクションに置く。理由: 投資戦略ごとに資金配分が異なるため、戦略と資金は同じファイルで管理するのが自然。

### Decision 2: Budget Context は Spec セクションとは別にプロンプトに注入する
Spec の raw テキストはそのまま保持し、Budget Context は別の `## Budget Context` セクションとしてプロンプトに追加する。理由: Spec の生テキスト方式を壊さない。budget 計算は動的なので静的な Spec テキストとは性質が異なる。

### Decision 3: 残高計算は trades テーブルから集計する
`remaining = initial_cash - Σ(買い約定額) + Σ(売り約定額)` で算出。portfolio_positions の評価額ではなく実際のキャッシュフローベースで計算する。理由: 購入可能な「現金」を把握するのが目的であり、含み益/損は関係ない。

### Decision 4: budget の抽出は toml::Table から行う
Spec は raw テキスト方式なので、budget.initial_cash の抽出も `toml::Table` の get で行う。型付き struct は作らない。

## Risks / Trade-offs

- [Risk] DB の trades と実際の証券口座残高が乖離する → Mitigation: ユーザーが initial_cash を実態に合わせて更新する運用。将来的にAPI連携で解消。
- [Risk] budget セクションがない既存 Spec で動作しない → Mitigation: budget が None の場合は Budget Context セクション自体をスキップ。
- [Trade-off] LLM への情報提供のみで強制力がない → 意図的な設計。execute フェーズでの資金チェックは別課題とする。
