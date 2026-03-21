## MODIFIED Requirements

### Requirement: 立花証券 API 認証
システムは SHALL 認証 I/F の AUTH_URL をハードコードの const ではなく `TachibanaConfig` の `environment` フィールドに基づいて決定する。

#### Scenario: 本番環境の認証
- **WHEN** environment が production の場合
- **THEN** `https://kabuka.e-shiten.jp/e_api_v4r8/auth/` に認証リクエストを送信する

#### Scenario: デモ環境の認証
- **WHEN** environment が demo の場合
- **THEN** `https://demo.e-shiten.jp/e_api_v4r8/auth/` に認証リクエストを送信する
