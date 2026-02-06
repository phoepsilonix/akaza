use std::fs::File;
use std::io::{self, BufRead, Write};
use std::sync::{Arc, Mutex};

use log::{error, info};
use serde::Serialize;

use libakaza::config::{Config, DictConfig, DictEncoding, DictType, DictUsage};
use libakaza::engine::bigram_word_viterbi_engine::{
    BigramWordViterbiEngine, BigramWordViterbiEngineBuilder,
};
use libakaza::graph::candidate::Candidate;
use libakaza::kana_kanji::base::KanaKanjiDict;
use libakaza::lm::base::{SystemBigramLM, SystemUnigramLM};
use libakaza::user_side_data::user_data::UserData;

#[derive(Debug, Serialize)]
struct JsonOutput {
    input: String,
    segments: Vec<SegmentOutput>,
    best_result: String,
    total_cost: f32,
}

#[derive(Debug, Serialize)]
struct SegmentOutput {
    yomi: String,
    candidates: Vec<CandidateOutput>,
}

#[derive(Debug, Serialize)]
struct CandidateOutput {
    surface: String,
    cost: f32,
}

pub struct CheckOptions<'a> {
    pub yomi: Option<String>,
    pub expected: Option<String>,
    pub use_user_data: bool,
    pub eucjp_dict: &'a [String],
    pub utf8_dict: &'a [String],
    pub model_dir: Option<&'a str>,
    pub json_output: bool,
    pub num_candidates: usize,
}

pub fn check(opts: CheckOptions) -> anyhow::Result<()> {
    // 設定ファイルを読み込む
    let mut config = Config::load()?;
    info!("Config loaded: model={}", config.engine.model);

    // モデルディレクトリが指定されていればオーバーライド
    if let Some(dir) = opts.model_dir {
        config.engine.model = dir.to_string();
        info!("Model directory overridden: {}", dir);
    }

    // 追加の辞書を設定に追加
    for path in opts.eucjp_dict {
        config.engine.dicts.push(DictConfig {
            dict_type: DictType::SKK,
            encoding: DictEncoding::EucJp,
            path: path.clone(),
            usage: DictUsage::Normal,
        });
    }

    for path in opts.utf8_dict {
        config.engine.dicts.push(DictConfig {
            dict_type: DictType::SKK,
            encoding: DictEncoding::Utf8,
            path: path.clone(),
            usage: DictUsage::Normal,
        });
    }

    // dict_cache を無効にする（開発用ツールなので）
    config.engine.dict_cache = false;

    let mut builder = BigramWordViterbiEngineBuilder::new(config.engine);

    if opts.use_user_data {
        info!("Enabled user data");
        match UserData::load_from_default_path() {
            Ok(ud) => {
                builder.user_data(Arc::new(Mutex::new(ud)));
            }
            Err(err) => {
                error!("Cannot load user data: {}", err);
            }
        }
    }

    let engine = builder.build()?;

    match opts.yomi {
        Some(yomi) => {
            // 引数指定モード: 従来通り1行処理
            check_one_line(
                &engine,
                &yomi,
                opts.expected,
                opts.json_output,
                opts.num_candidates,
            )?;
        }
        None => {
            // stdin モード: 行ごとに処理
            let stdin = io::stdin();
            for line in stdin.lock().lines() {
                let line = line?;
                if line.is_empty() {
                    continue;
                }
                check_one_line(&engine, &line, None, opts.json_output, opts.num_candidates)?;
            }
        }
    }

    Ok(())
}

fn check_one_line<U: SystemUnigramLM, B: SystemBigramLM, KD: KanaKanjiDict>(
    engine: &BigramWordViterbiEngine<U, B, KD>,
    yomi: &str,
    expected: Option<String>,
    json_output: bool,
    num_candidates: usize,
) -> anyhow::Result<()> {
    let lattice = engine.to_lattice(yomi, None)?;

    // DOT グラフ出力（expected が指定された場合）
    if let Some(expected) = expected {
        let dot = lattice.dump_cost_dot(expected.as_str());
        println!("{dot}");
        let mut file = File::create("/tmp/dump.dot")?;
        file.write_all(dot.as_bytes())?;
    }

    let mut result = engine.resolve(&lattice)?;

    // 候補数を制限する
    for segment in &mut result {
        segment.truncate(num_candidates);
    }

    if json_output {
        print_json(yomi, &result)?;
    } else {
        print_text(&result);
    }

    Ok(())
}

fn print_text(result: &[Vec<Candidate>]) {
    let text: Vec<String> = result
        .iter()
        .filter_map(|segment| segment.first().map(|c| c.surface_with_dynamic()))
        .collect();
    println!("{}", text.join("/"));
}

fn print_json(input: &str, result: &[Vec<Candidate>]) -> anyhow::Result<()> {
    let segments: Vec<SegmentOutput> = result
        .iter()
        .map(|segment| {
            let yomi = segment.first().map(|c| c.yomi.clone()).unwrap_or_default();
            let candidates: Vec<CandidateOutput> = segment
                .iter()
                .map(|c| CandidateOutput {
                    surface: c.surface_with_dynamic(),
                    cost: c.cost,
                })
                .collect();
            SegmentOutput { yomi, candidates }
        })
        .collect();

    let best_result: Vec<String> = result
        .iter()
        .filter_map(|segment| segment.first().map(|c| c.surface_with_dynamic()))
        .collect();

    let total_cost: f32 = result
        .iter()
        .filter_map(|segment| segment.first().map(|c| c.cost))
        .sum();

    let output = JsonOutput {
        input: input.to_string(),
        segments,
        best_result: best_result.join("/"),
        total_cost,
    };

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
