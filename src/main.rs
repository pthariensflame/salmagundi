use quote::*;
use std::{
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
    /// Type names to exclude in the randomization; accepts extended regular expressions with unicode support.
    exclude: Vec<String>,
}

pub struct Options {
    exclude: regex::RegexSet,
}

fn cmd_input_to_path(s: ffi::OsString) -> Option<path::PathBuf> {
    if s == "-" {
        None
    } else {
        Some(s.into())
    }
}

fn finish_parsing_options(CmdOpts { exclude }: CmdOpts) -> Result<(), Box<dyn std::error::Error>> {
    Options {
        exclude: regex::RegexSet::new(exclude),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
