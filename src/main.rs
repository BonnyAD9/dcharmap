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
use termal::{printc, printcln};

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
    let mut prev = Instant::now();

    let args = Args::parse(Pareg::args().get_ref())?;

    if args.exit() {
        return Ok(());
    }

    let mut now = Instant::now();
    eprintln!("Arg parse: {:?}", now - prev);
    prev = now;

    let fcm = if let Some(f) = args.dict {
        FcmData::new(args.words, BufReader::new(File::open(f)?).lines())?
    } else {
        FcmData::new(args.words, stdin().lock().lines())?
    };

    now = Instant::now();
    eprintln!("Initialization: {:?}", now - prev);
    prev = now;

    let solutions = if args.progress {
        printc!("{'save}");
        fcm.find_char_map(|p| {
            let eta = (Instant::now() - prev).as_secs_f32() * (1. - p) / p;
            printcln!("{'load e_}{:.2}% eta: {}s", p * 100., eta as i64);
        })
    } else {
        fcm.find_char_map(drop)
    };

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
