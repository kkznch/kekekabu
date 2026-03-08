## ADDED Requirements

### Requirement: 銘柄ごとの直近評価履歴を取得
システムは SHALL 指定された銘柄の直近 N 件の評価結果（decision, score, rationale, evaluated_at）を evaluations テーブルから取得する関数を提供する。

#### Scenario: 評価履歴が存在する銘柄
- **WHEN** 過去に5回評価された銘柄に対して直近3件の履歴を要求した場合
- **THEN** 最新3件の評価を evaluated_at 降順で返す

#### Scenario: 評価履歴が存在しない銘柄
- **WHEN** 過去に評価されたことがない銘柄に対して履歴を要求した場合
- **THEN** 空のリストを返す

#### Scenario: 履歴が要求件数より少ない銘柄
- **WHEN** 過去に2回しか評価されていない銘柄に対して直近3件の履歴を要求した場合
- **THEN** 存在する2件の評価を返す
