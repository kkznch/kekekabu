## Context

立花証券 e支店 API にはデモ環境（`demo.e-shiten.jp`）がある。本番環境とデモ環境で API 仕様は同一だが、エンドポイント URL が異なる。デモで発生したデータが本番に混入しないよう DB を分離する必要がある。

## Goals / Non-Goals

**Goals:**
- config またはCLI フラグでデモ/本番を切り替え可能にする
- デモ環境時は DB ファイルを分離する（`kekekabu-demo.db`）
- API エンドポイントを自動で切り替える

**Non-Goals:**
- デモ専用の機能追加（デモ環境は本番と同一 API 仕様）
- デモデータの自動投入

## Decisions

### Decision 1: Environment enum (Production / Demo)

`Environment` enum を定義し、config の `[tachibana] environment` フィールドで指定。CLI の `--demo` フラグで上書き可能。

```rust
pub enum Environment { Production, Demo }
```

### Decision 2: DB パスに environment を反映

`db_path()` に `Environment` を引数として渡し、Demo の場合は `kekekabu-demo.db` を返す。マイグレーションは両方の DB に対して同一スキーマを適用。

### Decision 3: AUTH_URL を const から config 由来に変更

`TachibanaClient` の AUTH_URL をハードコードの const から `TachibanaConfig` 経由で解決するように変更。

```
Production: https://kabuka.e-shiten.jp/e_api_v4r8/auth/
Demo:       https://demo.e-shiten.jp/e_api_v4r8/auth/
```

### Decision 4: --demo フラグは global option

`kabu --demo <command>` の形式で全コマンドに影響する。DB パスと API エンドポイントの両方が切り替わる。

## Risks / Trade-offs

- [Risk] デモ環境の URL が変更される可能性 → config で上書き可能にしておく
- [Trade-off] --demo を忘れて本番で操作 → execute は --live 必須のため、二重の安全策
