# k-best リランキング機構

## 背景

現在の Viterbi アルゴリズムでは、パスコストが `Σ(unigram_cost + bigram_cost)` の等重み合算で決まる。
bigram コストが支配的になりやすく、レアな bigram の異常値に引きずられて誤変換が発生するケースがある。

例:
- たまたまコーパスに「は→厚い」の bigram が多い → 文脈に関係なく「厚い」が勝つ
- bigram は直前の 1 単語しか見ないため、「夏は暑い」vs「板は厚い」を区別できない

## 方針: 特徴量ベースの線形リランキング

Viterbi（等重み）で k-best 候補を生成した後、重み付きスコアで再順位付けを行う。

```
Viterbi (等重み) → k-best 候補生成（多様な候補の探索に最適化）
       ↓
ReRanking (重み付き) → 最終順位（最終選択に最適化）
```

候補生成と最終選択で最適な重みが異なるのは自然であり、
Viterbi 側は等重みのまま維持して多様な候補を確保する。

### 初期フェーズの特徴量

まずは既存のコスト情報を分離するだけで始める:

```
rerank_score = unigram_weight × Σ unigram_cost + bigram_weight × Σ bigram_cost
```

- `unigram_weight`: デフォルト 1.0
- `bigram_weight`: デフォルト 1.0（= 従来と同じ挙動）

bigram_weight を 1.0 未満にすると、bigram のノイズが抑えられ、
単語自体の一般的な出現しやすさ（unigram）がアンカーとして効く。

### 将来の拡張

リランキングフレームワークが入れば、特徴量を追加するだけで拡張できる:

```
rerank_score = α × Σ unigram_cost
             + β × Σ bigram_cost
             + γ × trigram_score        # 将来
             + δ × embedding_score      # 将来
             + ε × rule_penalty         # 将来
```

検討中の追加特徴量:

| 特徴量 | 効果 | 実装コスト |
|---|---|---|
| Trigram スコア | 2 単語前まで文脈を拡張 | 中（新モデル構築が必要） |
| Skip-gram 埋め込み | 離れた単語間の意味的整合性 | 高（Word2Vec 学習 + スコアリング設計） |
| ルールベースペナルティ | 既知の誤パターンの抑制 | 低 |

#### Trigram vs Skip-gram の比較

| | Trigram | Skip-gram |
|---|---|---|
| 頻出パターンへの効果 | 高い | 高い |
| 未知の組み合わせへの汎化 | 弱い（スパース性の壁） | 強い（分散表現で汎化） |
| モデルサイズ | 大きくなりがち（bigram trie ~186MB の拡張） | 制御しやすい（語彙数 × 次元） |
| 既存コストとの統合 | 対数確率なので自然に加算 | コサイン類似度は確率ではないので重み調整が必要 |

現在のコーパス規模（Wikipedia + 青空文庫）では trigram のスパース性が厳しいため、
将来的には skip-gram の方が費用対効果は高いと予想される。

## 設計

### KBestPath の拡張

```rust
pub struct KBestPath {
    pub segments: Vec<Vec<Candidate>>,
    pub cost: f32,           // 従来の合算コスト（Viterbi DP 用）
    pub unigram_cost: f32,   // Σ unigram コスト
    pub bigram_cost: f32,    // Σ bigram コスト
}
```

### ReRankingWeights

```rust
pub struct ReRankingWeights {
    pub unigram_weight: f32,  // デフォルト 1.0
    pub bigram_weight: f32,   // デフォルト 1.0
}

impl ReRankingWeights {
    pub fn rerank(&self, paths: &mut Vec<KBestPath>) {
        for path in paths.iter_mut() {
            path.cost = self.unigram_weight * path.unigram_cost
                      + self.bigram_weight * path.bigram_cost;
        }
        paths.sort_by(|a, b| a.cost.partial_cmp(&b.cost).unwrap());
    }
}
```

### 重みの設定箇所

| 用途 | 設定方法 |
|---|---|
| `akaza-data check` | CLI: `--unigram-weight 1.0 --bigram-weight 0.7` |
| `akaza-data evaluate` | CLI: 同上。グリッドサーチで最適値を探索可能 |
| ibus-akaza | `config.yml` の `engine.reranking_weights` |

### 変更対象ファイル

- `libakaza/src/graph/graph_resolver.rs` — KBestPath にコスト内訳追加、forward DP で分離記録
- `libakaza/src/reranking.rs` (新規) — ReRankingWeights と rerank 関数
- `akaza-data/src/subcmd/check.rs` — CLI オプション追加
- `akaza-data/src/subcmd/evaluate.rs` — CLI オプション追加
- `libakaza/src/config.rs` — EngineConfig に reranking_weights 追加

## 期待される効果

### ポジティブな効果

1. **bigram ノイズの抑制**: bigram_weight を下げることで、レアな bigram の異常スコアに引きずられにくくなる
2. **一般的な単語の安定化**: unigram の重みが相対的に上がることで、一般的な単語がアンカーとして機能し、変換結果が安定する
3. **チューニングの容易化**: evaluate コーパスに対してグリッドサーチで最適重みを探索できる。従来は bigram/unigram の比率を変えるにはモデル再構築が必要だった
4. **拡張の基盤**: 将来の特徴量追加が容易になる

### 定量的な期待

- `akaza-data evaluate` の exact match rate が数ポイント改善する可能性がある
- 特に bigram がスパースな短い入力（2〜3 文節）で効果が出やすい
- bigram カバレッジが高い頻出パターンでは従来と同等の精度を維持

## 退行リスク

### リスク1: bigram を弱めすぎると同音異義語の判別力が低下

bigram は同音異義語の判別に本質的に効いている（例: 「板が厚い」vs「お湯が熱い」）。
bigram_weight を下げすぎると、直前の文脈が効かなくなり、これらのケースで退行する。

**対策**: evaluate コーパスの must.txt / should.txt で退行検知。デフォルト重みは (1.0, 1.0) で従来と同じ挙動を保証。

### リスク2: Viterbi の候補生成と最終順位の乖離

Viterbi は等重みで候補を生成するため、リランキング後に「本来 1 位になるべきパスが k-best に含まれていない」可能性がある。

**対策**: k の値を十分大きくする（5〜10）。evaluate で top-k hit rate を監視。

### リスク3: 重みの過学習

evaluate コーパスに過度にフィットした重みは、汎用的な変換で退行する可能性がある。

**対策**: コーパスを train/dev に分割して交差検証。極端な重み（0.0 や 10.0 等）にならないよう範囲を制限。

### リスク4: パフォーマンスへの影響

リランキング自体は k 個のパスを再ソートするだけなので計算コストはほぼゼロ。
ただし、KBestPath にフィールドが増えるため、メモリ使用量が微増する。

**影響**: 無視できるレベル。f32 × 2 フィールド追加のみ。

## 検証手順

1. `cargo test --all` で既存テストが pass することを確認
2. デフォルト重み (1.0, 1.0) で `akaza-data evaluate` の結果が変わらないことを確認
3. bigram_weight を 0.5〜0.9 で変えながら evaluate を実行し、精度の変化を観察
4. must.txt の全ケースで退行がないことを確認
5. `akaza-data check` で代表的な入力を手動確認
