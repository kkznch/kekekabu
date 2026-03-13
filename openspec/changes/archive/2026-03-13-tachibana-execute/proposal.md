## Why

execute コマンドは現在シグナル生成のみで、実際の注文発注・約定検知は「Tachibana API integration pending」のスタブ状態。立花証券 e支店 API のドキュメント調査が完了し、認証フロー・注文入力・EVENT I/F（WebSocket 約定通知）の仕様が判明したため、実装に着手できる段階になった。

## What Changes

- 立花証券 e支店 API クライアントモジュール（認証、REQUEST I/F、EVENT I/F WebSocket）を新規追加
- `orders` テーブルを新設し、発注した注文を pending 状態で追跡
- execute コマンドを拡充: settle（前回 pending 注文の約定確認）→ シグナル生成 → 指値発注 → 短時間 WebSocket 約定待ち
- 約定時に `portfolio::buy/sell` を呼び出してポジション・取引履歴を自動更新
- config に立花証券 API 接続情報（userId, password, secondPassword）を追加
- dry-run モードでは API 未接続でも従来どおりシグナルのみ出力

## Capabilities

### New Capabilities
- `tachibana-api`: 立花証券 e支店 API クライアント（認証、REQUEST I/F 注文入力/照会、EVENT I/F WebSocket 約定通知）
- `order-management`: 注文ライフサイクル管理（orders テーブル、ステータス遷移、べき等性の request_id）

### Modified Capabilities
- `trade-execution`: execute コマンドに settle フェーズ・実注文発注・約定検知を追加
- `database`: orders テーブル（10個目のテーブル）を追加
- `config`: 立花証券 API 接続情報のセクションを追加

## Impact

- `src/cmd/execute.rs` — 大幅改修（settle + 発注 + 約定待ちロジック）
- `src/tachibana/` — 新規モジュール（API クライアント）
- `src/db/schema.rs` — orders テーブル DDL 追加
- `src/db/mod.rs` — orders CRUD 操作追加
- `src/config.rs` — tachibana セクション追加
- `src/portfolio.rs` — execute から自動呼び出し（既存関数の活用）
- 依存追加: `tokio-tungstenite`（WebSocket）, `url`（URLエンコード）
