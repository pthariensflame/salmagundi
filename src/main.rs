#![feature(int_to_from_bytes)]

use quote::*;
use rand::{prelude::*, prng::XorShiftRng};
use regex::RegexSet;
use std::{
    error::Error,
    ffi, fs,
    io::{self, prelude::*},
    path,
};
use structopt::StructOpt;

pub mod rust;

pub struct FullFile {
    pub has_bom: bool,
    pub file: syn::File,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Default, StructOpt)]
#[structopt(name = "salmagundi")]
/// Rewrites data type definitions to rearrange in-memory layout.
struct CmdParams {
    #[structopt(value_name = "IN_FILE", parse(from_os_str))]
    /// A path to the file to read from; if not present or "-", use standard input.
    in_file_raw: Option<ffi::OsString>,
    #[structopt(
        short = "o",
        long = "out",
        value_name = "OUT_FILE",
        parse(from_os_str)
    )]
    /// A path to the file to write to; if not present or "-", use standard output.
    out_file_raw: Option<ffi::OsString>,
    #[structopt(short = "P", long = "passthrough")]
    /// Pass the input through unrandomized.
    passthrough: bool,
    #[structopt(flatten)]
    cmd_opts: CmdOpts,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Default, StructOpt)]
struct CmdOpts {
    #[structopt(short = "e", long = "exclude", value_name = "IDENTIFIER")]
    /// Type paths to exclude in the randomization; accepts extended regular expressions with unicode support.
    exclude: Vec<String>,
    #[structopt(short = "S", long = "seed", value_name = "SEED")]
    /// Numeric seed to use for reproducible randomization.
    seed: Option<u128>,
    #[structopt(short = "R", long = "print-seed")]
    /// Print the seed used for randomization to standard error.
    print_seed: bool,
}

#[derive(Clone, Debug)]
pub struct Options {
    pub exclude: RegexSet,
    pub rng: XorShiftRng,
}

fn cmd_input_to_path(s: ffi::OsString) -> Option<path::PathBuf> {
    if s == "-" {
        None
    } else {
        Some(s.into())
    }
}

fn finish_parsing_options(
    CmdOpts {
        exclude,
        seed,
        print_seed,
    }: CmdOpts,
) -> Result<Options, Box<dyn Error>> {
    Ok(Options {
        exclude: RegexSet::new(exclude)?,
        rng: XorShiftRng::from_seed({
            let s = seed.unwrap_or_else(|| random());
            if print_seed {
                eprintln!("seed = {}", s);
            }
            s.to_be_bytes()
        }),
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let CmdParams {
        in_file_raw,
        out_file_raw,
        passthrough,
        cmd_opts,
    } = CmdParams::from_args();
    let options = finish_parsing_options(cmd_opts)?;

    let stdin;
    let mut reader: Box<dyn Read + Sync> =
        if let Some(path) = in_file_raw.into_iter().flat_map(cmd_input_to_path).nth(0) {
            Box::<fs::File>::new(fs::File::open(path)?)
        } else {
            stdin = io::stdin();
            Box::<io::StdinLock>::new(stdin.lock())
        };
    let mut file_contents_raw = Vec::new();
    reader.read_to_end(&mut file_contents_raw)?;
    drop(reader);
    let file_contents = String::from_utf8(file_contents_raw)?;
    let mut full_file = FullFile {
        has_bom: file_contents.starts_with("\u{FEFF}"),
        file: syn::parse_file(&file_contents)?,
    };
    drop(file_contents);

    if !passthrough {
        rust::alter_file(&mut full_file.file, options)?;
    }

    let stdout;
    let mut writer: Box<dyn Write + Sync> =
        if let Some(path) = out_file_raw.into_iter().flat_map(cmd_input_to_path).nth(0) {
            Box::<fs::File>::new(fs::File::create(path)?)
        } else {
            stdout = io::stdout();
            Box::<io::StdoutLock>::new(stdout.lock())
        };
    if full_file.has_bom {
        write!(writer, "{}", "\u{FEFF}");
    }
    writeln!(writer, "{}", full_file.file.into_token_stream());
    drop(writer);

    Ok(())
}
