use log::info;

use crate::tokenizer::base::AkazaTokenizer;
use crate::tokenizer::vibrato::VibratoTokenizer;

/// 一行の自然文を `surface/yomi` 形式で出力する。
pub fn tokenize_line(
    system_dict: &str,
    user_dict: Option<String>,
    kana_preferred: bool,
    text: &str,
) -> anyhow::Result<()> {
    info!("tokenize-line: {}", text);
    let tokenizer = VibratoTokenizer::new(system_dict, user_dict)?;
    let annotated = tokenizer.tokenize(text, kana_preferred)?;
    println!("{annotated}");
    Ok(())
}
