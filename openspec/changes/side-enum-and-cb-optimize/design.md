## Context

execute / order 層で `"buy"` / `"sell"` が bare string として 20 箇所以上使われている。order.rs のワイルドカードマッチ `_ => "3"` は不正な side を黙って buy 扱いする金融リスクがあった（修正済み）。circuit_breaker は watchlist 全銘柄の全価格履歴（60日分 OHLCV）を 1 銘柄ずつ取得する N+1 パターンになっている。

## Goals / Non-Goals

**Goals:**
- Side enum で売買方向を型安全にし、コンパイル時にタイポを検出可能にする
- circuit_breaker のデータ取得を直近2件の終値のみに最適化する

**Non-Goals:**
- OrderType enum（成行/指値）の導入（別 change）
- circuit_breaker のバッチクエリ化（1クエリで全銘柄取得）は今回やらない

## Decisions

### Decision 1: Side enum を tachibana/mod.rs に定義

Side enum は注文に最も密接なため `tachibana/mod.rs` に定義する。`as_str()` と `Display` を実装し、DB 保存時は `side.as_str()` で文字列化する。DB から読み出す際は `"buy"` / `"sell"` → `Side` のパースは行わず、DB 層は引き続き `&str` で保存する（既存データとの互換性維持）。

**Alternative**: execute.rs やトップレベルに定義 → tachibana が最も自然な所有者なので却下。

### Decision 2: circuit_breaker 用に get_latest_closes メソッドを追加

`DbClient` trait に `get_latest_closes(stock_id: i64, n: usize) -> Result<Vec<f64>>` を追加する。`prices` テーブルから `ORDER BY date DESC LIMIT n` で直近 n 件の終値のみを取得し、日付昇順で返す。circuit_breaker は `n=2` で呼び出す。

**Alternative**: 既存の `fetch_price_data` にリミットパラメータ追加 → 既存の利用者（scan）に影響するため却下。

## Risks / Trade-offs

- [Risk] Side enum への移行で全テストが壊れる → 機械的な置換のため低リスク
- [Trade-off] DB 層は &str のまま → enum → string → enum の変換が発生するが、DB スキーマ変更を避ける
