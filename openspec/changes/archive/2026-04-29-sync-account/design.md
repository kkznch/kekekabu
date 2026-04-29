## Context

`kabu` の預かり金は spec の `initial_cash` から `total_invested + total_recovered` を加減して推定計算しているのみ。配当・手数料・税金・入出金は反映されない。さらに DB 上の `portfolio_positions` は約定通知ベースで更新するため、watch コマンドが動いていない時間帯の手動取引や API 障害で DB と実態が乖離するリスクがある。

立花証券 API には `CLMZanKaiKanougaku`（買付可能額）と `CLMGenbutuKabuList`（現物保有銘柄一覧）があり、これらを利用すれば実残高・実ポジションを取得できる。

## Goals / Non-Goals

**Goals:**
- 立花証券口座の実残高（買付可能額）を取得し DB に記録する
- 実建玉と DB の `portfolio_positions` を突合し、不整合を検出する
- `--fix` フラグで DB を実態に合わせて補正できる
- 残高履歴を保持し、配当・手数料の影響を可視化できる

**Non-Goals:**
- 信用取引の建玉同期（現物のみ対応、信用は将来対応）
- 過去の入出金履歴の自動取得（手動入力 or 立花証券側で履歴 API がない）
- リアルタイム残高同期（cron/launchd で定期同期する想定）

## Decisions

### Decision 1: 残高履歴を保持する `account_balance` テーブルを新設

**選択**: スナップショットを履歴として保持（時系列で増えるテーブル）

**理由**: 配当受領、手数料引き落としの累積影響を追跡するため。単一行で上書きすると変動の原因究明が困難。残高変動の可視化（グラフ等）にも将来対応しやすい。

**スキーマ**:
```sql
CREATE TABLE account_balance (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cash_available TEXT NOT NULL,    -- 買付可能額（円）
    synced_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

### Decision 2: ポジション突合は数量ベースで判定

**選択**: 数量の一致のみを比較。平均取得価格は突合しない

**理由**:
- 立花証券側の平均取得価格は税制上の調整（特定口座の譲渡損益計算）で kabu 側と異なる可能性がある
- 数量一致が業務上最もクリティカル（資金不足・空売りの直接原因）
- 平均取得価格の不一致は警告レベルにとどめ、自動補正は数量のみ

### Decision 3: `--fix` フラグでの自動補正は数量のみ

**選択**: `--fix` 指定時は DB の `quantity` を実建玉に合わせる。価格や `avg_cost` は変えない

**理由**:
- 数量だけ合っていれば次回 execute 時の判断は正しくなる
- avg_cost を勝手に書き換えると過去の取引履歴と整合しなくなる
- 真に乖離している場合は手動で `db reset` して再構築する想定

### Decision 4: BrokerClient trait に新メソッドを追加

**選択**: `query_balance()` と `query_positions()` を `BrokerClient` trait に追加

**理由**:
- 既存の DI パターンに合わせる（execute と同じ構造）
- テスト時の MockBrokerClient で動作検証可能
- 将来別ブローカー対応する際の拡張点になる

### Decision 5: sync コマンドは `--demo` も対応

**選択**: `kabu --demo sync` でデモ環境の同期も可能にする

**理由**: デモ環境でも動作検証できる。本番投入前の動作確認が容易。

## Risks / Trade-offs

- [Risk] 立花証券 API の障害時に sync が失敗 → リトライロジックを実装、失敗時は warn ログのみで実害なし
- [Risk] 手動取引が大量にあった場合 `--fix` で DB と乖離 → ポジション差分を必ずログ出力し、`--fix` は明示指定でのみ動作
- [Trade-off] 信用取引未対応 → 現物のみのユーザーには影響なし、信用は別 issue で対応
- [Trade-off] 配当・手数料の自動分類はしない → 残高履歴の差分から推定可能、必要なら別コマンドで対応
