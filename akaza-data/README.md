# akaza-data

辞書および言語モデルの管理用ツールです。

## インストール

```bash
cargo build --package akaza-data --release
```

## コマンド一覧

### check - かな漢字変換を実行する

ひらがな文字列をかな漢字変換して結果を表示します。

```bash
# 基本的な使い方（設定ファイルのデフォルト設定を使用）
akaza-data check きょうはいいてんきですね
# => 今日/は/いい/天気/です/ね

# JSON 出力
akaza-data check --format json きょうはいいてんきですね

# 複数候補を表示
akaza-data check --format json --candidates 3 きょうはいいてんきですね

# ユーザー学習データを使用
akaza-data check --user-data きょうはいいてんきですね

# モデルディレクトリを指定
akaza-data check --model-dir /path/to/model きょうはいいてんきですね
```

### evaluate - 変換精度を評価する

コーパスを使って変換精度を評価します。

```bash
akaza-data evaluate --corpus corpus.txt --model-dir /path/to/model
```

### tokenize - コーパスをトーカナイズする

コーパスを形態素解析器でトーカナイズします。

```bash
akaza-data tokenize --reader wikipedia --system-dict /path/to/unidic src_dir dst_dir
```

### wfreq - 単語頻度ファイルを生成する

トーカナイズされたコーパスから単語頻度ファイルを生成します。

```bash
akaza-data wfreq --src-dir tokenized_corpus output.wfreq
```

### vocab - 語彙リストを生成する

単語頻度ファイルから語彙リストを生成します。

```bash
akaza-data vocab --threshold 10 input.wfreq output.vocab
```

### make-dict - システム辞書を作成する

語彙リストとコーパスからシステム辞書を作成します。

```bash
akaza-data make-dict --corpus corpus_dir --unidic /path/to/unidic --vocab vocab.txt output.txt
```

### wordcnt-unigram - ユニグラム言語モデルを作成する

単語頻度ファイルからユニグラム言語モデルを作成します。

```bash
akaza-data wordcnt-unigram input.wfreq output.model
```

### wordcnt-bigram - バイグラム言語モデルを生成する

コーパスからバイグラム言語モデルを生成します。

```bash
akaza-data wordcnt-bigram --threshold 5 --corpus-dirs corpus_dir unigram.model bigram.model
```

### learn-corpus - 言語モデルを学習する

コーパスから言語モデルのパラメータを学習します。

```bash
akaza-data learn-corpus --delta 1 may.corpus should.corpus must.corpus \
    src_unigram.model src_bigram.model dst_unigram.model dst_bigram.model
```

### dump-unigram-dict / dump-bigram-dict - 辞書をダンプする

言語モデルの内容をテキスト形式でダンプします。

```bash
akaza-data dump-unigram-dict unigram.model
akaza-data dump-bigram-dict unigram.model bigram.model
```

## 詳細なヘルプ

各コマンドの詳細なオプションは `--help` で確認できます。

```bash
akaza-data check --help
akaza-data evaluate --help
```
