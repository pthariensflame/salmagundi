#![feature(int_to_from_bytes)]

use either::*;
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

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FullFile {
    pub has_bom: bool,
    pub file: Either<syn::File, String>,
}

/// Potentially-supported source languages.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Language {
    /// Rust
    Rust,
    /// A C-like language
    CLike(CLikeLanguage),
}

impl Default for Language {
    fn default() -> Self { Language::Rust }
}

/// Potentially-supported C-like source languages.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum CLikeLanguage {
    /// C
    C,
    /// C++
    Cxx,
    /// Objective-C
    ObjC,
    /// Objective-C++
    ObjCxx,
}

impl Default for CLikeLanguage {
    fn default() -> Self { CLikeLanguage::C }
}

pub fn try_parse_language(s: &str) -> Result<Language, Box<dyn Error>> {
    Ok(match s
        .to_lowercase()
        .replace(|c: char| c.is_whitespace() || c == '-', "")
        .as_ref()
    {
        "" | "default" => Language::default(),
        "rust" => Language::Rust,
        "clike" | "clikedefault" | "cfamily" | "cfamilydefault" | "cfam" | "cfamdefault" => {
            Language::CLike(CLikeLanguage::default())
        }
        "c" => Language::CLike(CLikeLanguage::C),
        "cpp" | "cxx" | "c++" => Language::CLike(CLikeLanguage::Cxx),
        "objectivec" | "objc" => Language::CLike(CLikeLanguage::ObjC),
        "objectivecpp" | "objectivecxx" | "objectivec++" | "objcpp" | "objcxx" | "objc++" => {
            Language::CLike(CLikeLanguage::ObjCxx)
        }
        other => Err(format!("Unsupported source language \"{}\"", other))?,
    })
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Default, StructOpt)]
#[structopt(name = "salmagundi")]
/// Rewrites data type definitions to rearrange in-memory layout.
struct CmdParams {
    #[structopt(value_name = "IN_FILE", parse(from_os_str))]
    /// A path to the file to read from; if not present or "-", use standard
    /// input.
    in_file_raw: Option<ffi::OsString>,
    #[structopt(
        short = "o",
        long = "out",
        value_name = "OUT_FILE",
        parse(from_os_str)
    )]
    /// A path to the file to write to; if not present or "-", use standard
    /// output.
    out_file_raw: Option<ffi::OsString>,
    #[structopt(short = "P", long = "passthrough")]
    /// Pass the input through unrandomized.
    passthrough: bool,
    #[structopt(
        short = "L",
        long = "language",
        parse(try_from_str = "try_parse_language"),
        value_name = "LANGUAGE",
        case_insensitive = true,
        default_value = ""
    )]
    /// Source language of the code to transform.
    language: Language,
    #[structopt(flatten)]
    cmd_opts: CmdOpts,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Default, StructOpt)]
struct CmdOpts {
    #[structopt(short = "e", long = "exclude", value_name = "IDENTIFIER")]
    /// Type path(s) to exclude in the randomization even if they would
    /// otherwise be excluded; takes precedence over the explicit "include"
    /// option; accepts extended regular expressions with unicode support.
    exclude: Vec<String>,
    #[structopt(short = "i", long = "include", value_name = "IDENTIFIER")]
    /// Type path(s) to include in the randomization even if they would
    /// implicitly be excluded; takes precedence over any implicit exclusions,
    /// but not over the explicit "exclude" option; accepts extended regular
    /// expressions with unicode support.
    include: Vec<String>,
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
    pub include: RegexSet,
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
        include,
        seed,
        print_seed,
    }: CmdOpts,
) -> Result<Options, Box<dyn Error>> {
    Ok(Options {
        exclude: RegexSet::new(exclude)?,
        include: RegexSet::new(include)?,
        rng: XorShiftRng::from_seed({
            let s = seed.unwrap_or_else(random);
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
        language,
        cmd_opts,
    } = CmdParams::from_args();
    let options = finish_parsing_options(cmd_opts)?;

    // TODO: this is ugly; give the user a better error.
    assert_eq!(
        language,
        Language::Rust,
        "Currently, only Rust source code is supported."
    );

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
        file: Left(syn::parse_file(&file_contents)?),
    };
    drop(file_contents);

    if !passthrough {
        rust::alter_file(
            full_file
                .file
                .as_mut()
                .left()
                .unwrap_or_else(|| unreachable!()),
            options,
        )?;
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
        write!(writer, "\u{FEFF}");
    }
    // I wish this could use "format_args!" but there were borrowck errors.  :(
    let full_output = full_file.file.map_left(|mut t| {
        format!(
            "{}{}",
            t.shebang.take().map_or(Left(""), |s| Right(s + "\n")),
            t.into_token_stream()
        )
    });
    writeln!(writer, "{}", full_output);
    drop(writer);

    Ok(())
}
