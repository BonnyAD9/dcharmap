use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write, stdin, stdout},
    process::ExitCode,
    time::Instant,
};

use anyhow::Result;
use args::Args;
use find_char_map::{FcmData, WordTree};
use pareg::Pareg;

mod args;
mod find_char_map;
mod help;
mod notree;

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
    let mut now = Instant::now();

    let args = Args::parse(Pareg::args().get_ref())?;

    if args.exit() {
        return Ok(());
    }

    let mut prev = now;
    now = Instant::now();
    eprintln!("Arg parse: {:?}", now - prev);

    let fcm = if let Some(f) = args.dict {
        FcmData::new(args.words, BufReader::new(File::open(f)?).lines())?
    } else {
        FcmData::new(args.words, stdin().lock().lines())?
    };

    prev = now;
    now = Instant::now();
    eprintln!("Initialization: {:?}", now - prev);

    let solutions = fcm.find_char_map();

    prev = now;
    now = Instant::now();
    eprintln!("Search: {:?}", now - prev);

    if let Some(out) = args.out {
        print_solutions(
            &solutions,
            fcm.word_map(),
            BufWriter::new(File::create(out)?),
        );
    } else {
        print_solutions(&solutions, fcm.word_map(), stdout().lock());
    }

    prev = now;
    now = Instant::now();
    eprintln!("Printing: {:?}", now - prev);

    Ok(())
}

fn print_solutions(
    solutions: &[WordTree],
    map: &[usize],
    mut out: impl Write,
) {
    for s in solutions {
        s.walk(|s| {
            for i in map {
                //std::hint::black_box(i);
                _ = write!(out, "{} ", &s[*i]);
            }
            _ = writeln!(out);
        });
    }
}
