## Context

eval の LLM 出力は非決定的で、同じ入力でも判断が揺れる。現状 temperature 制御もプロンプト/レスポンスのログも存在しないため、出力の再現性確保やデバッグが困難。

現在の LlmBackend トレイトは `send_message(prompt, max_tokens)` と `send_message_with_schema(...)` の2メソッドを持ち、temperature パラメータは渡せない。API バックエンド（api-anthropic, api-gemini）は temperature をリクエストボディに含める機能を持つが、CLI バックエンド（cli-claude, cli-gemini）は temperature 制御不可。

## Goals / Non-Goals

**Goals:**

- eval 呼び出し時に temperature=0 を指定して出力の揺れを抑制する
- LLM のプロンプト/レスポンスを DB に保存して事後検証を可能にする
- `kabu show llm-logs` で保存されたログを閲覧可能にする

**Non-Goals:**

- LLM コスト（トークン数・費用）の追跡（別変更で対応）
- fetch / discover の temperature 制御（eval のみ対象）
- CLI バックエンドへの temperature 強制（不可能なため）

## Decisions

### 1. temperature パラメータの追加方法

LlmBackend トレイトのメソッドシグネチャに `temperature: Option<f32>` を追加する。

**代替案: ビルダーパターン / config フィールド**
バックエンド構築時に temperature を固定する方法も検討したが、同じバックエンドを fetch（temperature なし）と eval（temperature=0）で使い分ける必要があるため、メソッドレベルで渡す方が柔軟。

`send_message` と `send_message_with_schema` の両方に追加する。`None` の場合は API デフォルトを使用する。

### 2. CLI バックエンドの温度制御

CLI バックエンドは temperature 制御をサポートしないため、`temperature: Some(...)` が指定された場合に `warn!` ログを出して無視する。

### 3. LLM ログの保存先

`llm_logs` テーブルを新設する。evaluations とは独立して全 LLM 呼び出しを記録する。

```sql
CREATE TABLE IF NOT EXISTS llm_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    command TEXT NOT NULL,       -- 'eval', 'fetch', 'discover'
    ticker TEXT,                 -- nullable (discover は銘柄なし)
    backend TEXT NOT NULL,       -- 'api-anthropic', 'cli-claude', etc.
    model TEXT,
    temperature REAL,
    prompt TEXT NOT NULL,
    response TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

**代替案: ファイルベースのログ**
ファイルに書き出す方法も検討したが、既に SQLite を中心に据えた設計のため DB に統一する方がクエリ・管理が容易。

### 4. ログ保存の責務

LlmBackend トレイト内部ではなく、呼び出し元（eval, fetch, discover）でログを保存する。トレイトを DB 非依存に保つため。

### 5. show llm-logs コマンド

`kabu show llm-logs` で直近のログを閲覧できるようにする。`--limit` と `--ticker` フィルタをサポート。

## Risks / Trade-offs

- **[DB 容量増加]** → prompt/response は長文になりうる。定期的な古いログの削除は運用で対応（この変更では自動パージ不実装）
- **[CLI バックエンド非対応]** → temperature 無視を warn ログで通知。ユーザーが CLI バックエンドを eval に使う場合は非決定性が残る
- **[トレイトの破壊的変更]** → 全バックエンド実装の更新が必要だが、4つしかないため影響は限定的
