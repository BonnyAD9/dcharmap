use std::{
    borrow::Cow,
    io::{IsTerminal, stdout},
};

use termal::{gradient, printmcln};

pub fn help() {
    let color = stdout().is_terminal();
    let version = option_env!("CARGO_PKG_VERSION").unwrap_or("unknown");
    let sign: Cow<str> = if color {
        gradient("BonnyAD9", (250, 50, 170), (180, 50, 240)).into()
    } else {
        "BonnyAD9".into()
    };

    printmcln!(
        color,
        "Welcome in help for {'i g}dcharmap{'_} by {sign}
Version: {version}

{'i g}dcharmap{'_} can discover char mapping so that all words are in
from dictionary.

{'g}Usage:
  {'m}ddcharmap {'gr}[{'dy}flags{'gr}] [--] [{'dy}words{'gr}]

{'g}Flags:
  {'y}-h  -?  --help{'_}
    Show this help.

  {'y}-d  --dictionary{'_}
    Specify the dictionary file. By default this is stdin.

  {'y}-o  --output{'_}
    Specify output file. If not specified, write to stdout.

  {'y}-p  --progress{'_}
    Show progress.

  {'y}-w  --word{'_}
    Add word. This word may start with `{'i}-{'_}`.

  {'y}--{'_}
    Read the remaining arguments as words.
"
    );
}
