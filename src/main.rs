use quote::*;
use rand::prelude::*;
use rayon::prelude::*;
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
struct CmdOpts {
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
    #[structopt(
        short = "e",
        long = "exclude",
        value_name = "IDENTIFIER",
        parse(try_from_str = "syn::parse_str")
    )]
    /// Types to exclude in the randomization.
    exclude: Vec<syn::Ident>,
}

fn cmd_input_to_path(s: ffi::OsString) -> Option<path::PathBuf> {
    if s == "-" {
        None
    } else {
        Some(s.into())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let CmdOpts {
        in_file_raw,
        out_file_raw,
        passthrough,
        exclude,
    } = CmdOpts::from_args();

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
    let full_file = FullFile {
        has_bom: file_contents.starts_with("\u{FEFF}"),
        file: syn::parse_file(&file_contents)?,
    };
    drop(file_contents);

    if !passthrough {
        // TODO: implement file alteration
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
