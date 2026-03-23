## MODIFIED Requirements

### Requirement: 売買シグナルの生成と発注
システムは SHALL execute コマンド内のシグナル（Signal 構造体）および注文結果（OrderResult 構造体）の売買方向フィールドとして `Side` enum を使用する。execute は注文発注後に即座に結果を返し、WebSocket による約定待受は行わない。約定の検知は `watch` コマンドまたは次回 execute の settle フェーズに委ねる。

#### Scenario: 注文発注後の即時 return
- **WHEN** execute が非 dry-run モードで注文を発注した場合
- **THEN** 注文番号を含む結果を返して即座に終了する。WebSocket 接続は行わない

#### Scenario: settle による約定検知
- **WHEN** execute の Phase 1（settle）で pending 状態の注文が存在する場合
- **THEN** ORDER I/F（CLMOrderListDetail）で約定状態を照会し、DB を更新する
