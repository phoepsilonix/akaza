use anyhow::Result;
use half::f16;
use log::info;

use rsmarisa::{Agent, Keyset, Trie};

use crate::lm::base::SystemSkipBigramLM;

/*
   {word1 ID}    # 3 bytes (w_{i-2})
   {word2 ID}    # 3 bytes (w_i)
   packed float  # score: 2 bytes (f16)
*/

/// skip-bigram 言語モデルのビルダー。
/// bigram LM と同一キー形式 `[3B id1][3B id2][2B f16_score]` を使用。
pub struct MarisaSystemSkipBigramLMBuilder {
    keyset: Keyset,
}

impl Default for MarisaSystemSkipBigramLMBuilder {
    fn default() -> Self {
        Self {
            keyset: Keyset::new(),
        }
    }
}

impl MarisaSystemSkipBigramLMBuilder {
    pub fn add(&mut self, word_id1: i32, word_id2: i32, score: f32) {
        let id1_bytes = word_id1.to_le_bytes();
        let id2_bytes = word_id2.to_le_bytes();

        assert_eq!(id1_bytes[3], 0);
        assert_eq!(id2_bytes[3], 0);

        let mut key: Vec<u8> = Vec::new();
        key.extend(id1_bytes[0..3].iter());
        key.extend(id2_bytes[0..3].iter());
        key.extend(f16::from_f32(score).to_le_bytes());
        self.keyset.push_back_bytes(&key, 1.0).unwrap();
    }

    pub fn build(&mut self) -> Result<MarisaSystemSkipBigramLM> {
        let mut trie = Trie::new();
        trie.build(&mut self.keyset, 0);
        Ok(MarisaSystemSkipBigramLM { trie })
    }

    pub fn save(&mut self, ofname: &str) -> Result<()> {
        let mut trie = Trie::new();
        trie.build(&mut self.keyset, 0);
        trie.save(ofname)?;
        Ok(())
    }
}

pub struct MarisaSystemSkipBigramLM {
    trie: Trie,
}

impl MarisaSystemSkipBigramLM {
    pub fn load(filename: &str) -> Result<MarisaSystemSkipBigramLM> {
        info!("Loading system-skip-bigram: {}", filename);
        let mut trie = Trie::new();
        trie.load(filename)?;
        Ok(MarisaSystemSkipBigramLM { trie })
    }

    /// パス上の word_id 列から skip-bigram コストの合計を計算する。
    /// skip-bigram は w_{i-2} と w_i のペア。
    pub fn compute_path_cost(&self, word_ids: &[Option<i32>]) -> f32 {
        let mut total = 0.0_f32;
        for i in 2..word_ids.len() {
            if let (Some(id1), Some(id2)) = (word_ids[i - 2], word_ids[i]) {
                if let Some(cost) = self.get_skip_cost(id1, id2) {
                    total += cost;
                }
            }
        }
        total
    }
}

impl SystemSkipBigramLM for MarisaSystemSkipBigramLM {
    fn get_skip_cost(&self, word_id1: i32, word_id2: i32) -> Option<f32> {
        let id1_bytes = word_id1.to_le_bytes();
        let id2_bytes = word_id2.to_le_bytes();
        let key: [u8; 6] = [
            id1_bytes[0],
            id1_bytes[1],
            id1_bytes[2],
            id2_bytes[0],
            id2_bytes[1],
            id2_bytes[2],
        ];

        let mut agent = Agent::new();
        agent.set_query_bytes(&key);

        if self.trie.predictive_search(&mut agent) {
            let keyword = agent.key().as_bytes();
            if keyword.len() < 2 {
                return None;
            }
            let last2: [u8; 2] = match keyword[keyword.len() - 2..keyword.len()].try_into() {
                Ok(bytes) => bytes,
                Err(_) => return None,
            };
            let score: f16 = f16::from_le_bytes(last2);
            return Some(score.to_f32());
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_and_lookup() -> anyhow::Result<()> {
        let mut builder = MarisaSystemSkipBigramLMBuilder::default();
        builder.add(100, 200, 3.5);
        builder.add(100, 300, 4.0);
        let lm = builder.build()?;

        let cost = lm.get_skip_cost(100, 200).unwrap();
        assert!(3.4 < cost && cost < 3.6);

        let cost = lm.get_skip_cost(100, 300).unwrap();
        assert!(3.9 < cost && cost < 4.1);

        assert!(lm.get_skip_cost(999, 888).is_none());
        Ok(())
    }

    #[test]
    fn compute_path_cost_basic() -> anyhow::Result<()> {
        let mut builder = MarisaSystemSkipBigramLMBuilder::default();
        builder.add(1, 3, 2.0);
        let lm = builder.build()?;

        // word_ids: [Some(1), Some(2), Some(3)]
        // skip-bigram: (1, 3) at i=2
        let cost = lm.compute_path_cost(&[Some(1), Some(2), Some(3)]);
        assert!(1.9 < cost && cost < 2.1);

        // 見つからないペアは寄与なし
        let cost = lm.compute_path_cost(&[Some(10), Some(20), Some(30)]);
        assert_eq!(cost, 0.0);

        // None が含まれる場合はスキップ
        let cost = lm.compute_path_cost(&[Some(1), None, Some(3)]);
        assert!(1.9 < cost && cost < 2.1);

        // 短いパスでは skip-bigram なし
        let cost = lm.compute_path_cost(&[Some(1), Some(3)]);
        assert_eq!(cost, 0.0);

        Ok(())
    }
}
