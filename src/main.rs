use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufRead, BufReader, stdin},
    process::ExitCode,
};

use anyhow::Result;
use args::Args;
use find_char_map::find_char_map;
use itertools::Itertools;
use pareg::Pareg;

mod args;
mod find_char_map;
mod help;

fn main() -> ExitCode {
    match start() {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

fn start() -> Result<()> {
    let args = Args::parse(Pareg::args().get_ref())?;

    if args.exit() {
        return Ok(());
    }

    let mut words = args.words.iter().unique().cloned().collect_vec();
    words.sort_unstable_by_key(|s| usize::MAX - s.len());

    let lengths: HashSet<_> = words.iter().map(|a| a.len()).unique().collect();
    let filter = |s: &str| lengths.contains(&s.len());

    let dict = if let Some(f) = args.dict {
        read_dict(BufReader::new(File::open(f)?), filter)?
    } else {
        read_dict(stdin().lock(), filter)?
    };

    let solutions = find_char_map(&words, &dict);

    for s in solutions {
        s.walk(|s| {
            for w in &args.words {
                let word =
                    &s[words.iter().find_position(|a| *a == w).unwrap().0];
                print!("{word} ")
            }
            println!()
        });
    }

    Ok(())
}

fn read_dict(
    input: impl BufRead,
    mut filter: impl FnMut(&str) -> bool,
) -> Result<HashMap<usize, Vec<String>>> {
    let mut res = HashMap::new();

    for l in input.lines() {
        let l = l?;
        let l = l.trim();

        if !filter(l) {
            continue;
        }

        res.entry(l.len())
            .or_insert_with(Vec::new)
            .push(l.to_string());
    }

    Ok(res)
}
