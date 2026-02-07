use std::ops::Range;

use crate::graph::candidate::Candidate;

pub trait HenkanEngine {
    fn learn(&mut self, candidates: &[Candidate]);

    fn convert(
        &self,
        yomi: &str,
        force_ranges: Option<&[Range<usize>]>,
    ) -> anyhow::Result<Vec<Vec<Candidate>>>;

    /// k-best ビタビで上位 k 個の分節パターンを返す。
    /// 外側がパス（分節パターン）、中が文節、内が漢字候補。
    fn convert_k_best(
        &self,
        yomi: &str,
        force_ranges: Option<&[Range<usize>]>,
        k: usize,
    ) -> anyhow::Result<Vec<Vec<Vec<Candidate>>>>;
}
