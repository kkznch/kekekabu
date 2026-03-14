## Context

`kabu` CLI は SQLite DB を使用し、refinery でスキーマ管理を行っている。現状 DB の管理操作（マイグレーション状態確認、リセット等）は CLI から直接行えず、開発者が手動で DB ファイルを操作する必要がある。

既存の `SqliteClient::open()` は起動時に自動マイグレーションを行うが、適用されたマイグレーションの確認手段がない。また、DB をリセットする安全な方法も提供されていない。

## Goals / Non-Goals

**Goals:**
- `kabu db migrate` でマイグレーションを明示的に実行し、適用状況を確認できるようにする
- `kabu db status` で DB のパス・サイズ・マイグレーション履歴を確認できるようにする
- `kabu db reset` でランダム確認コードによる安全な DB 削除を提供する

**Non-Goals:**
- マイグレーションのロールバック機能
- 個別マイグレーションの選択的適用
- DB バックアップ機能

## Decisions

### Decision 1: DB サブコマンドは DB open の前に処理する

DB サブコマンド（特に reset）は DB 接続前に処理する必要がある。reset は DB ファイル自体を削除するため、open() でマイグレーションを適用してから削除するのは無意味。main.rs の早期 return パターンを使用し、config の読み込み前に処理する。

**Alternative**: DB open 後に処理する → reset で不要なマイグレーション実行が発生するため却下。

### Decision 2: reset の確認にランダム6文字コードを使用する

単純な "yes" 確認ではなく、ランダム生成される6文字の英数字コードを入力させる。これにより、ユーザーが実際にターミナルを見て意識的に操作していることを保証する。`--force` フラグでスキップ可能（スクリプト利用時向け）。

**Alternative**: "yes" 入力 → 誤操作リスクが高いため却下。DB 名入力 → パスが長すぎて不便なため却下。

### Decision 3: MigrationInfo は SqliteClient のメソッドとして提供

`migration_status()` は `SqliteClient` の直接メソッドとして実装し、`DbClient` trait には含めない。マイグレーション状態の確認は運用管理機能であり、ビジネスロジックからの利用は想定しない。

**Alternative**: `DbClient` trait に含める → テスト時のモック実装が不要な機能を強制するため却下。

### Decision 4: WAL/SHM ファイルの同時削除

reset 時に DB ファイル本体に加えて `.db-wal` と `.db-shm` ファイルも削除する。SQLite WAL モードのアーティファクトが残ると、再作成時に不整合が発生する可能性がある。

## Risks / Trade-offs

- [Risk] reset で誤ってデータ削除 → ランダム確認コードと警告メッセージで軽減
- [Risk] migrate 実行時に既に open() で自動マイグレーション済み → refinery が冪等にスキップするため問題なし
- [Trade-off] `migration_status()` が `DbClient` trait 外 → DI の一貫性は損なわれるが、運用ツール専用機能として割り切る
