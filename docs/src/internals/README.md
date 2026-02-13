# 内部構造

このセクションでは、Akaza の内部構造と設計に関する技術文書を掲載しています。

## 目次

- [データフロー](data-flow.md) — 言語モデル構築と辞書生成の流れ
- [K-Best セグメンテーション](k-best-segmentation.md) — Tab キーによる複数分節パターンの切り替え
- [文節伸縮の仕様](clause-extension-behavior.md) — Shift+矢印キーによる文節の伸縮操作
- [リランキング](reranking.md) — K-Best パスの特徴量ベースリランキング
- [リランキング評価レポート](reranking-evaluation-report.md) — リランキングの評価結果
- [構造化パーセプトロン](structured-perceptron.md) — 識別学習によるパラメータチューニング
- [ユーザーデータ](user-data.md) — ユーザー固有のデータ管理
- 前処理
  - [漢方薬の読み問題](preproc/kanpoyaku.md) — kytea の分割誤り対応
  - [MeCab 形態素解析](preproc/mecab.md) — 形態素解析の粒度問題
