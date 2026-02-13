# Akaza

Akaza は、Rust で書かれた統計的かな漢字変換エンジンを搭載した Linux 向け日本語 IME（IBus エンジン）です。

## 特徴

- **Rust で実装**: UI/Logic をすべて Rust で書いてあるので、拡張が容易です
- **統計的かな漢字変換**: 単語 bigram モデルを採用し、Wikipedia・青空文庫から構築した言語モデルで変換を行います
- **学習機能**: ユーザーの変換結果を学習し、使い込むほど変換精度が向上します
- **SKK 辞書対応**: SKK 形式の辞書ファイルを複数読み込み可能
- **GUI 設定ツール**: GTK4 ベースの設定ツール (`akaza-conf`, `akaza-dict`) を提供

## リンク

- [GitHub リポジトリ](https://github.com/akaza-im/akaza)
- [デフォルトモデル](https://github.com/akaza-im/akaza-default-model)
