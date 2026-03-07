## 1. spec.rs の書き換え

- [x] 1.1 InvestmentSpec を name + raw_content のみの構造体に変更
- [x] 1.2 load_spec() を toml::Table でパースし name のみ抽出する実装に変更
- [x] 1.3 to_prompt_section() を生 TOML テキストをコードブロックで返す実装に変更
- [x] 1.4 不要な構造体（UniverseFilter, ScoringConfig, ScoringFactor, ExecutionConfig）と validate() を削除

## 2. テスト

- [x] 2.1 新方式のテストを追加（TOML パース、name 抽出、不正 TOML、name 欠落、自由構造の TOML）
- [x] 2.2 cargo test で全テスト通過を確認
- [x] 2.3 cargo clippy で警告がないことを確認

## 3. 検証

- [x] 3.1 実際の jp-core-value-quality-v1.toml で config validate が通ることを確認
- [x] 3.2 template.toml でも config validate が通ることを確認

## 4. ドキュメント

- [x] 4.1 README.md の投資 Spec セクションを更新（構造は自由、name のみ必須の説明）
