## Why

立花証券 e支店 API にはデモ環境（`demo.e-shiten.jp`）があり、本番口座を使わずにAPI動作を検証できる。デモ環境で発生した注文データが本番データに混ざらないよう、DB を分離して安全にテストできる仕組みが必要。

## What Changes

- config に `[tachibana] environment = "demo" | "production"` を追加（デフォルト: production）
- デモ環境時は AUTH_URL を `https://demo.e-shiten.jp/e_api_v4r8/auth/` に切り替え
- デモ環境時は DB パスを `~/.config/kabu/kekekabu-demo.db` に切り替え
- CLI に `--demo` フラグを追加（config の environment を上書き可能）

## Capabilities

### New Capabilities
- `demo-mode`: 本番/デモ環境の切り替え機能（API エンドポイント + DB 分離）

### Modified Capabilities
- `database`: DB パスの決定ロジックに environment 分岐を追加
- `tachibana-api`: AUTH_URL を environment に応じて切り替え

## Impact

- `src/config.rs` — TachibanaConfig に `environment` フィールド追加
- `src/db/mod.rs` — `db_path()` に environment 引数追加
- `src/tachibana/mod.rs` — AUTH_URL をハードコードから config 参照に変更
- `src/main.rs` — `--demo` グローバルフラグ追加、DB パス分岐
