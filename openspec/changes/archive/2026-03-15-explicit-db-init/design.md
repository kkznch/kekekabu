## Context

`SqliteClient::open()` は `rusqlite::Connection::open()` を呼んでおり、SQLite の仕様上ファイルが存在しなければ自動作成される。これを防ぐため、`open()` の冒頭でファイル存在チェックを入れる。

## Goals / Non-Goals

**Goals:**
- DB ファイルが存在しない場合、`open()` はエラーメッセージ付きで失敗する
- `kabu db migrate` だけが DB を新規作成できる
- 既存の DB がある場合の挙動は変えない（マイグレーション自動適用は維持）

**Non-Goals:**
- DB のバックアップ/リストア機能
- DB パスの動的変更

## Decisions

### Decision 1: open() と open_or_create() の分離

`open()` は DB ファイル存在を前提とし、なければエラー。`open_or_create()` は親ディレクトリ作成 + DB 作成 + マイグレーション適用を行う。共通のマイグレーションロジックは `open_and_migrate()` に抽出。

**Alternative**: `open()` にフラグ引数を追加 → メソッド名で意図を表現する方が安全なため却下。

### Decision 2: main.rs の変更は不要

main.rs の L218 で `SqliteClient::open()` を呼んでおり、config/db/service 以外の全コマンドがこのチェックを通る。既存のフロー構造で十分。

## Risks / Trade-offs

- [Risk] 初回利用時に「DB がない」エラーで混乱 → エラーメッセージに `kabu db migrate` の案内を含めて軽減
