## Purpose

本番環境とデモ環境を安全に切り替える機能。API エンドポイントと DB を分離し、デモデータが本番に混入するリスクを排除する。

## Requirements

### Requirement: 環境の切り替え
システムは SHALL config の `[tachibana] environment` フィールドまたは CLI の `--demo` フラグでデモ/本番環境を切り替える。

#### Scenario: config による環境指定
- **WHEN** config に `environment = "demo"` が設定されている場合
- **THEN** デモ環境の API エンドポイントとデモ用 DB を使用する

#### Scenario: --demo フラグによる上書き
- **WHEN** `kabu --demo <command>` を実行した場合
- **THEN** config の environment 設定に関わらずデモ環境を使用する

#### Scenario: デフォルトは本番
- **WHEN** environment が未指定の場合
- **THEN** 本番環境（production）をデフォルトとして使用する

### Requirement: API エンドポイントの切り替え
システムは SHALL environment に応じて立花証券 API のエンドポイント URL を切り替える。

#### Scenario: 本番環境
- **WHEN** environment が production の場合
- **THEN** `https://kabuka.e-shiten.jp/e_api_v4r8/auth/` を使用する

#### Scenario: デモ環境
- **WHEN** environment が demo の場合
- **THEN** `https://demo.e-shiten.jp/e_api_v4r8/auth/` を使用する

### Requirement: DB パスの分離
システムは SHALL environment に応じて異なる DB ファイルを使用する。

#### Scenario: 本番環境の DB
- **WHEN** environment が production の場合
- **THEN** `~/.config/kabu/kekekabu.db` を使用する

#### Scenario: デモ環境の DB
- **WHEN** environment が demo の場合
- **THEN** `~/.config/kabu/kekekabu-demo.db` を使用する
