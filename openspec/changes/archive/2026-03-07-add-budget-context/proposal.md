## Why

discover/eval の LLM プロンプトに資金状況が渡されておらず、予算を無視した銘柄推薦や評価が行われる。30万円規模の少額投資では、100株単元で購入可能な価格帯の考慮が不可欠。

## What Changes

- Spec TOML に `[budget]` セクションを追加し `initial_cash` を定義可能にする
- DB の trades テーブルから買い/売り合計を集計し、残り投資可能額を算出する関数を追加
- discover / eval のプロンプトに Budget Context セクションを注入し、LLM が資金状況を考慮して判断できるようにする
- 将来の証券API連携時に、DB計算をAPI残高に差し替え可能な設計とする

## Capabilities

### New Capabilities
- `budget`: 投資資金の管理と残高計算。Spec の initial_cash と DB の取引履歴から投資可能額を算出し、LLM プロンプトに注入する。

### Modified Capabilities
- `stock-discovery`: discover プロンプトに Budget Context セクションを追加
- `investment-evaluation`: eval プロンプトに Budget Context セクションを追加

## Impact

- `spec.rs`: InvestmentSpec から budget.initial_cash を抽出するロジック追加
- `db/mod.rs`: trades テーブルから買い/売り合計額を集計するクエリ追加
- `cmd/discover.rs`: プロンプト組み立てに Budget Context 注入
- `cmd/eval.rs`: プロンプト組み立てに Budget Context 注入
- ユーザーの Spec TOML: `[budget]` セクション追加（任意、なければ Budget Context をスキップ）
