## Context

立花証券 API v4r8 公式リファレンス（`mfds_json_api_ref_text.html`）との突合で、`order.rs` のフィールド名が独自命名になっていることが判明。compress/uncompress は実装済みだが、フィールド名自体が公式と異なるためサーバーが認識できない。

## Goals / Non-Goals

**Goals:**
- CLMKabuNewOrder のフィールド名を公式 v4r8 に完全準拠させる
- sSecondPassword を注文に含める
- CLMOrderListDetail のレスポンスパースを公式フィールド名に合わせる

**Non-Goals:**
- EVENT I/F のサブスクリプション形式修正（別 change）
- 信用取引・逆指値の完全対応（フィールドは追加するが「指定なし」のデフォルト値を使う）

## Decisions

### Decision 1: second_password は BrokerClient trait 経由で渡す

`place_order` のシグネチャに `second_password: &str` を追加する。TachibanaClient は config から取得して渡す。MockBrokerClient は空文字を受け取る。

**Alternative**: order.rs 内で直接 config を参照 → DI パターンに反するため却下。

### Decision 2: sZyoutoekiKazeiC は「特定」固定

現物取引のデフォルトとして `"1"`（特定口座）を使用。NISA 対応は将来の別 change で。

### Decision 3: レスポンスのフィールド名は公式準拠に統一

`parse_order_detail_value` で使うフィールド名も公式に合わせる。既存テストは新しいフィールド名で書き直す。

## Risks / Trade-offs

- [Risk] 公式フィールド名にも undocumented なものがある可能性 → デモ環境で実際のレスポンスを確認して対処
- [Trade-off] second_password を place_order の引数にするとインタフェースが太くなる → trait の一貫性のために許容
