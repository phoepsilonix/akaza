const COUNTER_SURFACES_HIKI: [&str; 1] = ["匹"];
const COUNTER_SURFACES_SHUKAN: [&str; 1] = ["週間"];

fn fullwidth_to_ascii(ch: char) -> Option<char> {
    if ('０'..='９').contains(&ch) {
        Some(char::from_u32((ch as u32) - ('０' as u32) + ('0' as u32)).unwrap())
    } else {
        None
    }
}

fn is_kanji_numeral_char(ch: char) -> bool {
    matches!(
        ch,
        '零' | '〇'
            | '一'
            | '二'
            | '三'
            | '四'
            | '五'
            | '六'
            | '七'
            | '八'
            | '九'
            | '十'
            | '百'
            | '千'
            | '万'
            | '億'
            | '兆'
            | '京'
            | '垓'
            | '𥝱'
            | '穣'
            | '溝'
            | '澗'
            | '正'
            | '載'
            | '極'
            | '恒'
            | '河'
            | '沙'
            | '阿'
            | '僧'
            | '祇'
            | '那'
            | '由'
            | '他'
            | '不'
            | '可'
            | '思'
            | '議'
            | '無'
            | '量'
            | '大'
            | '数'
    )
}

fn leading_numeric_prefix_len_surface(s: &str) -> usize {
    let mut end = 0;
    for (idx, ch) in s.char_indices() {
        if ch.is_ascii_digit() || fullwidth_to_ascii(ch).is_some() || is_kanji_numeral_char(ch) {
            end = idx + ch.len_utf8();
        } else {
            break;
        }
    }
    end
}

fn leading_numeric_prefix_len_reading(s: &str) -> usize {
    let mut end = 0;
    for (idx, ch) in s.char_indices() {
        if ch.is_ascii_digit() || fullwidth_to_ascii(ch).is_some() {
            end = idx + ch.len_utf8();
        } else {
            break;
        }
    }
    if end > 0 {
        return end;
    }
    for pat in [
        "ひゃく",
        "ひゃっ",
        "ぜろ",
        "れい",
        "じゅう",
        "せん",
        "きゅう",
        "さん",
        "よん",
        "なな",
        "ろく",
        "はち",
        "ご",
        "に",
        "いち",
    ] {
        if s.starts_with(pat) {
            return pat.len();
        }
    }
    0
}

pub fn normalize_counter_yomi(yomi: &str) -> Option<&'static str> {
    match yomi {
        "ひき" | "びき" | "ぴき" => Some("ひき"),
        "しゅうかん" => Some("しゅうかん"),
        _ => None,
    }
}

pub fn counter_surfaces_for(canonical_yomi: &str) -> Option<&'static [&'static str]> {
    match canonical_yomi {
        "ひき" => Some(&COUNTER_SURFACES_HIKI),
        "しゅうかん" => Some(&COUNTER_SURFACES_SHUKAN),
        _ => None,
    }
}

/// 助数詞の user 学習を数字に依存しない形で集約するためのキー正規化。
///
/// 例:
/// - "3匹/3びき" -> "<NUM>匹/<NUM>ひき"
/// - "５１６週間/516しゅうかん" -> "<NUM>週間/<NUM>しゅうかん"
/// - "五百十六週間/516しゅうかん" -> "<NUM>週間/<NUM>しゅうかん"
pub fn normalize_counter_key_for_lm(key: &str) -> Option<String> {
    let slash_pos = key.find('/')?;
    let surface = &key[..slash_pos];
    let reading = &key[slash_pos + 1..];

    let surface_prefix_len = leading_numeric_prefix_len_surface(surface);
    let reading_prefix_len = leading_numeric_prefix_len_reading(reading);
    if surface_prefix_len == 0 || reading_prefix_len == 0 {
        return None;
    }

    let surface_suffix = &surface[surface_prefix_len..];
    let reading_suffix = &reading[reading_prefix_len..];
    if surface_suffix.is_empty() || reading_suffix.is_empty() {
        return None;
    }

    let canonical_yomi = normalize_counter_yomi(reading_suffix)?;
    let allowed_surfaces = counter_surfaces_for(canonical_yomi)?;
    if !allowed_surfaces.contains(&surface_suffix) {
        return None;
    }

    Some(format!("<NUM>{surface_suffix}/<NUM>{canonical_yomi}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_counter_key_for_lm() {
        assert_eq!(
            normalize_counter_key_for_lm("3匹/3びき"),
            Some("<NUM>匹/<NUM>ひき".to_string())
        );
        assert_eq!(
            normalize_counter_key_for_lm("５１６週間/516しゅうかん"),
            Some("<NUM>週間/<NUM>しゅうかん".to_string())
        );
        assert_eq!(
            normalize_counter_key_for_lm("五百十六週間/516しゅうかん"),
            Some("<NUM>週間/<NUM>しゅうかん".to_string())
        );
        assert_eq!(
            normalize_counter_key_for_lm("0匹/ぜろひき"),
            Some("<NUM>匹/<NUM>ひき".to_string())
        );
        assert_eq!(normalize_counter_key_for_lm("3人/3にん"), None);
    }
}
