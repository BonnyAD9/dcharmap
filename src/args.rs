use anyhow::Result;
use pareg::ParegRef;

use crate::help::help;

#[derive(Debug, Default)]
pub struct Args {
    pub dict: Option<String>,
    pub words: Vec<String>,
    pub exit: bool,
    pub out: Option<String>,
    pub progress: bool,
}

impl Args {
    pub fn parse(mut args: ParegRef) -> Result<Self> {
        let mut res = Self::default();

        while let Some(arg) = args.next() {
            match arg {
                "-h" | "-?" | "--help" => {
                    help();
                    res.exit = true;
                }
                "-d" | "--dictionary" => res.dict = Some(args.next_arg()?),
                "-w" | "--word" => res.words.push(args.next_arg()?),
                "-o" | "--output" => res.out = args.next_arg()?,
                "-p" | "--progress" => res.progress = true,
                "--" => res
                    .words
                    .extend(args.remaining().iter().map(|a| a.to_string())),
                v if v.starts_with('-') => {
                    args.err_unknown_argument().err()?
                }
                _ => res.words.push(args.cur_arg()?),
            }
        }

        Ok(res)
    }

    pub fn exit(&self) -> bool {
        self.exit && self.dict.is_none() && self.words.is_empty()
    }
}
