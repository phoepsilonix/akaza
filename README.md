# Akaza (ibus-akaza)

Yet another kana-kanji-converter on IBus, written in Rust.

統計的かな漢字変換による日本語IMEです。
Rust で書いています。

**現在、開発途中のプロダクトです。非互換の変更が予告なくはいります**

[![CI](https://github.com/akaza-im/akaza/actions/workflows/ci-simple.yml/badge.svg)](https://github.com/akaza-im/akaza/actions/workflows/ci-simple.yml)

## 関連プロジェクト

* [akaza-default-model](https://github.com/akaza-im/akaza-default-model) - デフォルト言語モデル
* [jawiki-kana-kanji-dict](https://github.com/tokuhirom/jawiki-kana-kanji-dict) - Wikipedia ベース SKK 辞書

## モチベーション

いじりやすくて **ある程度** UIが使いやすいかな漢字変換があったら面白いなと思ったので作ってみています。
「いじりやすくて」というのはつまり、Hack-able であるという意味です。

モデルデータを自分で生成できて、特定の企業に依存しない自由なかな漢字変換エンジンを作りたい。

## 特徴

* **Rust で実装**: UI/Logic をすべて Rust で書いてあるので、拡張が容易です
* **統計的かな漢字変換**: 2gram 言語モデルを採用
    * 言語モデルの生成元は日本語 Wikipedia と青空文庫です
    * 形態素解析器 [Vibrato](https://github.com/daac-tools/vibrato) で分析した結果をもとに構築
    * 利用者の環境で 1 から言語モデルを再生成することが可能です
* **学習機能**: ユーザー環境で、利用者の変換結果を学習します (unigram, bigram の頻度を学習)
* **GUI 設定ツール**: GTK4 ベースの設定ツールを提供
    * `akaza-conf`: キーマップ、辞書、モデルなどの設定
    * `akaza-dict`: ユーザー辞書の編集
* **SKK 辞書対応**: SKK 形式の辞書ファイルを複数読み込み可能

## Dependencies

### Runtime dependencies

* ibus 1.5+
* marisa-trie (libmarisa)
* GTK 4.0+ (設定ツール用)

### Build time dependencies

* Rust 1.92.0+ (stable toolchain)
* Cargo
* pkg-config
* clang
* libibus-1.0-dev
* libmarisa-dev
* libgtk-4-dev
* libgirepository1.0-dev

### Supported environment

* **OS**: Linux 6.0 以上
* **IBus**: 1.5 以上

## Install 方法

### 1. ビルド依存関係のインストール

Ubuntu/Debian の場合:

```bash
sudo apt-get update
sudo apt-get install ibus libgirepository1.0-dev libmarisa-dev clang libibus-1.0-dev libgtk-4-dev pkg-config
```

### 2. Rust のインストール

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup install stable
```

### 3. ibus-akaza のビルドとインストール

```bash
git clone https://github.com/akaza-im/akaza.git
cd akaza
make
sudo make install
```

`make install` により、モデルファイル（[akaza-default-model](https://github.com/akaza-im/akaza-default-model)）のダウンロードとインストールも自動で行われます。

### 4. IBus の再起動と有効化

```bash
ibus restart
ibus engine akaza
```

または、IBus の設定画面から Akaza を追加してください。

## 設定方法

### GUI 設定ツール（推奨）

インストール後、以下のコマンドで GUI 設定ツールを起動できます：

```bash
# 一般設定（キーマップ、辞書、モデルなど）
akaza-conf

# ユーザー辞書の編集
akaza-dict
```

GUI ツールを使用すると、キーマップの選択、SKK 辞書の追加、モデルの切り替えなどが簡単に行えます。

### 手動設定（上級者向け）

設定ファイルは `~/.config/akaza/config.yml` に保存されます。

#### Keymap の設定

Akaza は典型的には以下の順番で探します。

1. `~/.local/share/akaza/keymap/{KEYMAP_NAME}.yml`
2. `/usr/local/share/akaza/keymap/{KEYMAP_NAME}.yml`
3. `/usr/share/akaza/keymap/{KEYMAP_NAME}.yml`

このパスは、[XDG ユーザーディレクトリ](https://wiki.archlinux.jp/index.php/XDG_%E3%83%A6%E3%83%BC%E3%82%B6%E3%83%BC%E3%83%87%E3%82%A3%E3%83%AC%E3%82%AF%E3%83%88%E3%83%AA)
の仕様に基づいています。
Akaza は Keymap を `XDG_DATA_HOME` と `XDG_DATA_DIRS` から探します。
`XDG_DATA_HOME` は設定していなければ `~/.local/share/` です。`XDG_DATA_DIRS` は設定していなければ `/usr/local/share:/usr/share/` です。

#### RomKan の設定

ローマ字かなマップも同様のパスから探します。

1. `~/.local/share/akaza/romkan/{ROMKAN_NAME}.yml`
2. `/usr/local/share/akaza/romkan/{ROMKAN_NAME}.yml`
3. `/usr/share/akaza/romkan/{ROMKAN_NAME}.yml`

設定変更は `akaza-conf` の GUI で行うことを推奨します。

#### Model の設定

Model は複数のファイルからなります：

- `unigram.model`
- `bigram.model`
- `SKK-JISYO.akaza`

以下のパスから読み込まれます：

- `~/.local/share/akaza/model/{MODEL_NAME}/unigram.model`
- `~/.local/share/akaza/model/{MODEL_NAME}/bigram.model`
- `~/.local/share/akaza/model/{MODEL_NAME}/SKK-JISYO.akaza`

keymap, romkan と同様に、`XDG_DATA_DIRS` からも読むことができます。

## FAQ

### 最近の言葉が変換できません/固有名詞が変換できません

流行り言葉が入力できない場合、[jawiki-kana-kanji-dict](https://github.com/tokuhirom/jawiki-kana-kanji-dict) の利用を検討してください。
Wikipedia から自動的に抽出されたデータを元に SKK 辞書を作成しています。
Github Actions で自動的に実行されているため、常に新鮮です。

一方で、自動抽出しているために変なワードも入っています。変なワードが登録されていることに気づいたら、github issues で報告してください。

### 人名が入力できません

必要な SKK 辞書を読み込んでください。

**GUI での設定（推奨）**:
1. `akaza-conf` を起動
2. 「辞書」タブから SKK 辞書ファイルを追加

**手動での設定**:
`~/.config/akaza/config.yml` の `skk_dicts` セクションに辞書パスを追加してください。

利用可能な SKK 辞書: https://skk-dev.github.io/dict/

## プロジェクト構成

このリポジトリは以下のクレートで構成されています：

* **ibus-akaza**: IBus エンジン本体
* **libakaza**: かな漢字変換エンジンのコアロジック
* **akaza-conf**: GUI 設定ツール (GTK4)
* **akaza-dict**: GUI 辞書編集ツール (GTK4)
* **akaza-data**: 言語モデル生成ツール
* **ibus-sys**: IBus の Rust バインディング

## 開発

### ビルド

```bash
# すべてのクレートをビルド
cargo build --workspace

# リリースビルド
cargo build --workspace --release

# テスト実行
cargo test --workspace
```

### コードフォーマット

```bash
cargo fmt --all
```

### Lint

```bash
cargo clippy -- -D warnings
```

## ライセンス

MIT License

## THANKS TO

* [ibus-uniemoji](https://github.com/salty-horse/ibus-uniemoji) を参考に初期の実装を行いました
* [日本語入力を支える技術](https://gihyo.jp/book/2012/978-4-7741-4993-6) を読み込んで実装しました。この本がなかったら実装しようと思わなかったと思います
* 形態素解析器 [Vibrato](https://github.com/daac-tools/vibrato) を使用しています

