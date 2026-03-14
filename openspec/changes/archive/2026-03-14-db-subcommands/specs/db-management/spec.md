## Purpose

CLI からのデータベース管理操作を提供する。マイグレーション実行・状態確認・データベースリセットの3つのサブコマンドで、開発時やトラブルシューティング時の DB 管理を安全かつ簡便に行えるようにする。

## Requirements

### Requirement: マイグレーション実行と状態表示
システムは SHALL `kabu db migrate` コマンドでデータベースのマイグレーションを実行し、適用済みマイグレーションの一覧を表示する。

#### Scenario: マイグレーション実行
- **WHEN** `kabu db migrate` を実行した場合
- **THEN** DB を open（自動マイグレーション適用）し、適用済みマイグレーションの一覧（version, name, applied_on）を JSON または human 形式で出力する

### Requirement: データベース状態表示
システムは SHALL `kabu db status` コマンドでデータベースのパス・ファイルサイズ・マイグレーション履歴を表示する。

#### Scenario: 正常な状態表示
- **WHEN** DB ファイルが存在する状態で `kabu db status` を実行した場合
- **THEN** DB パス、ファイルサイズ（バイト）、適用済みマイグレーション一覧を JSON または human 形式で出力する

#### Scenario: DB ファイルが存在しない場合
- **WHEN** DB ファイルが存在しない状態で `kabu db status` を実行した場合
- **THEN** エラーメッセージを表示して異常終了する

### Requirement: データベースリセット
システムは SHALL `kabu db reset` コマンドでランダム確認コードによる対話式の安全な DB 削除を提供する。

#### Scenario: 対話式リセット
- **WHEN** `kabu db reset` を実行した場合
- **THEN** ランダム生成された6文字の英数字確認コードを表示し、ユーザーが正しく入力した場合のみ DB ファイル（本体・WAL・SHM）を削除する

#### Scenario: 確認コード不一致
- **WHEN** ユーザーが誤った確認コードを入力した場合
- **THEN** 「Code mismatch. Aborting.」と表示して DB を削除せずに終了する

#### Scenario: 強制リセット
- **WHEN** `kabu db reset --force` を実行した場合
- **THEN** 確認コードの入力を求めず、直ちに DB ファイル（本体・WAL・SHM）を削除する

#### Scenario: DB ファイルが存在しない場合のリセット
- **WHEN** DB ファイルが存在しない状態で `kabu db reset` を実行した場合
- **THEN** エラーメッセージを表示して異常終了する

### Requirement: DB サブコマンドのルーティング
システムは SHALL DB サブコマンドを config 読み込みや通常の DB open よりも前に処理する。reset は DB 接続不要、migrate/status は独自に DB を open する。

#### Scenario: main.rs でのルーティング順序
- **WHEN** `kabu db <subcommand>` が実行された場合
- **THEN** config 読み込みの前に DB サブコマンドをディスパッチし、処理後に早期 return する
