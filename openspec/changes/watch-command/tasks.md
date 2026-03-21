## 1. watch コマンドの実装

- [ ] 1.1 `src/cmd/watch.rs` を作成（login → WebSocket 接続 → EC イベントループ → DB 更新）
- [ ] 1.2 `src/cmd/mod.rs` に `pub mod watch` を追加
- [ ] 1.3 `src/main.rs` に `Watch` コマンドバリアントを追加しルーティング

## 2. 約定通知の DB 記録

- [ ] 2.1 watch 内で EC イベント受信時に `update_order_and_record_fill` を呼び出す
- [ ] 2.2 既に filled の注文への二重処理防止チェックを追加

## 3. 再接続ロジック

- [ ] 3.1 WebSocket 切断時の指数バックオフ再接続を実装（1s → 2s → 4s → ... → 60s）
- [ ] 3.2 再接続時にログインからやり直すロジックを追加

## 4. execute からの Phase 7 削除

- [ ] 4.1 `execute.rs` から Phase 7（WebSocket fill wait）のコードを削除
- [ ] 4.2 `BrokerClient` trait から `wait_for_fills` メソッドを削除
- [ ] 4.3 テストを更新

## 5. シグナルハンドリング

- [ ] 5.1 SIGINT（Ctrl-C）でのグレースフルシャットダウン（logout + WebSocket close）を実装

## 6. 検証

- [ ] 6.1 `just ci` で全テスト通過確認
