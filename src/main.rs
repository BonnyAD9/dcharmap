use std::{
    fs::File,
    io::{BufRead, BufReader, stdin},
    process::ExitCode,
};

use anyhow::Result;
use args::Args;
use find_char_map::FcmData;
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

    let fcm = if let Some(f) = args.dict {
        FcmData::new(args.words, BufReader::new(File::open(f)?).lines())?
    } else {
        FcmData::new(args.words, stdin().lock().lines())?
    };

    let solutions = fcm.find_char_map();

    for s in solutions {
        s.walk(|s| {
            for i in fcm.word_map() {
                print!("{} ", &s[*i]);
            }
            println!();
        });
    }

    Ok(())
}
