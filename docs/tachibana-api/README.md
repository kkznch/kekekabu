# 立花証券 e支店 API ドキュメント

立花証券 e支店 API の調査結果と参考資料をまとめたディレクトリ。

## ファイル一覧

| ファイル | 内容 |
|---------|------|
| [api_reference_v4r8.md](api_reference_v4r8.md) | v4r8 API リファレンス（認証・REQUEST/EVENT I/F・注文・照会・マスタ等の全仕様）|
| [api-summary.md](api-summary.md) | API 調査サマリー（アーキテクチャ、約定検知、orders テーブル設計案）|
| [order-status-codes.md](order-status-codes.md) | 注文ステータスコード一覧 |
| [faq.md](faq.md) | 公式 FAQ（Q&A）|
| [archive-v4r7/](archive-v4r7/) | v4r7 旧資料（PDF + テキスト抽出版）|

## 公式リソース

- API トップページ: https://www.e-shiten.jp/api/
- API 仕様・マニュアル: https://www.e-shiten.jp/e_api/mfds_json_api_menu.html
- GitHub サンプルコード: https://github.com/e-shiten-jp
- デモ環境: https://www.e-shiten.jp/Service/demo.html
- FAQ: https://www.e-shiten.jp/QA/answer14.html

## API バージョン

- 最新: v4r8（2025-09-27 リリース）
- v4r7: 2025-11-29 廃止
- WebSocket 版 EVENT I/F は v4r7 で追加
- v4r8 の変更点: Auth I/F と REQUEST I/F で HTTPS POST をサポート
