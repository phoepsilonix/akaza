use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use log::{info, warn};
use rayon::prelude::*;
use regex::Regex;
use rustc_hash::FxHashMap;

use crate::utils::get_file_list;

/// 単語の発生確率を数え上げる。
pub fn wfreq(src_dirs: &Vec<String>, dst_file: &str) -> anyhow::Result<()> {
    info!("wfreq: {:?} => {}", src_dirs, dst_file);

    let mut file_list: Vec<PathBuf> = Vec::new();
    for src_dir in src_dirs {
        let list = get_file_list(Path::new(src_dir))?;
        for x in list {
            file_list.push(x)
        }
    }

    // fold + reduce で逐次マージし、メモリ使用量をスレッド数分に抑える。
    // 従来は全ファイル分の HashMap を collect してからマージしていたため、
    // 大規模コーパスで OOM が発生していた。
    let merged: FxHashMap<String, u32> = file_list
        .par_iter()
        .fold(
            || FxHashMap::default(),
            |mut acc: FxHashMap<String, u32>, path_buf| {
                info!("Processing {} for wfreq", path_buf.to_string_lossy());
                let file = match File::open(path_buf) {
                    Ok(f) => f,
                    Err(e) => {
                        warn!("Failed to open {}: {}", path_buf.to_string_lossy(), e);
                        return acc;
                    }
                };
                for line in BufReader::new(file).lines() {
                    let line = match line {
                        Ok(l) => l,
                        Err(_) => continue,
                    };
                    let line = line.trim();
                    for word in line.split(' ') {
                        if word.is_empty()
                            || word.as_bytes()[0] == b'/'
                            || word.as_bytes()[0] == b' '
                        {
                            continue;
                        }
                        if word.contains('\u{200f}') {
                            warn!("The document contains RTL character");
                            continue;
                        }
                        *acc.entry(word.to_string()).or_insert(0) += 1;
                    }
                }
                acc
            },
        )
        .reduce(
            || FxHashMap::default(),
            |mut a, b| {
                for (word, cnt) in b {
                    *a.entry(word).or_insert(0) += cnt;
                }
                a
            },
        );

    // 最終結果ファイルは順番が安定な方がよいので BTreeMap を採用。
    info!("Merging into sorted map");
    let mut retval: BTreeMap<String, u32> = BTreeMap::new();
    for (word, cnt) in merged {
        *retval.entry(word).or_insert(0) += cnt;
    }

    // 結果をファイルに書いていく
    info!("Write to {}", dst_file);
    // 明らかに不要なワードが登録されているのを除外する。
    // カタカナ二文字系は全般的にノイズになりがちだが、Wikipedia/青空文庫においては
    // 架空の人物や実在の人物の名前として使われがちなので、消す。
    let re = Regex::new("^[\u{30A0}-\u{30FF}]{2}/[\u{3040}-\u{309F}]{2}$")?;
    // let ignore_files = HashSet::from(["テル/てる", "ニナ/にな", "ガチ/がち"]);
    let mut ofp = File::create(dst_file.to_string() + ".tmp")?;
    for (word, cnt) in retval {
        if re.is_match(word.as_str()) {
            info!("Skip 2 character katakana entry: {}", word);
            continue;
        }
        ofp.write_fmt(format_args!("{word}\t{cnt}\n"))?;
    }
    fs::rename(dst_file.to_owned() + ".tmp", dst_file)?;

    Ok(())
}
