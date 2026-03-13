# 注文ステータスコード (sOrderStatusCode)

出典: CLMOrderListDetail レスポンス仕様

## 通常注文

| Code | 状態 | 英語名 | 備考 |
|------|------|--------|------|
| 0 | 受付未済 | Pending | 立花証券システムに到達、未処理 |
| 1 | 未約定 | Open | 市場に出ているが未約定 |
| 2 | 受付エラー | Rejected | 注文が拒否された |
| 3 | 訂正中 | Modifying | 注文訂正処理中 |
| 4 | 訂正完了 | Modified | 注文訂正完了 |
| 5 | 訂正失敗 | ModifyFailed | 注文訂正失敗 |
| 6 | 取消中 | Cancelling | 注文取消処理中 |
| 7 | 取消完了 | Cancelled | 注文取消完了 |
| 8 | 取消失敗 | CancelFailed | 注文取消失敗 |
| 9 | 一部約定 | PartialFill | 部分的に約定 |
| 10 | 全部約定 | Filled | 全量約定 |
| 11 | 一部失効 | PartialExpired | 一部が失効 |
| 12 | 全部失効 | Expired | 全量失効（当日限り注文の場合等）|
| 13 | 発注待ち | Queued | 発注待ち状態 |
| 14 | 無効 | Invalid | 無効 |
| 15 | 切替注文 | Switching | 切替注文中 |
| 16 | 切替完了 | SwitchDone | 切替注文完了 |
| 17 | 切替注文失敗 | SwitchFailed | 切替注文失敗 |
| 19 | 繰越失効 | CarryOverExpired | 繰越失効 |
| 20 | 一部障害処理 | PartialError | 一部障害処理 |
| 21 | 障害処理 | Error | 障害処理 |

## 逆指値・通常+逆指値注文

| Code | 状態 | 備考 |
|------|------|------|
| 15 | 逆指注文(切替中) | |
| 16 | 逆指注文(未約定) | |
| 17 | 逆指注文(失敗) | |
| 50 | 発注中 | |

## kekekabu での orders.status マッピング案

```
API sOrderStatusCode  →  orders.status
─────────────────────────────────────
0  (受付未済)          →  pending
1  (未約定)            →  pending (open)
9  (一部約定)          →  partial
10 (全部約定)          →  filled
2  (受付エラー)        →  rejected
7  (取消完了)          →  cancelled
12 (全部失効)          →  expired
19 (繰越失効)          →  expired
```
