//! Parse every Rust source file below a directory into a `syn::File` and write a debug
//! representation of each syntax tree to an output directory, mirroring the source tree:
//! `src/component/identifier.rs` is written to `<output>/component/identifier.rs.ast`.
//!
//! Use the following command from this directory to test this program by
//! running it on its own source code:
//!
//!     cargo run -- src output
//!
//! `output/main.rs.ast` will then begin with:
//!
//!     File {
//!         shebang: None,
//!         attrs: [
//!             Attribute {
//!                 pound_token: Pound,
//!                 style: AttrStyle::Inner(
//!         ...
//!     }

use colored::Colorize;
use std::borrow::Cow;
use std::env;
use std::ffi::OsStr;
use std::fmt::{self, Display};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;

enum Error {
    IncorrectUsage,
    ReadDirectory(io::Error),
    ReadFile(io::Error),
    WriteFile(io::Error),
    ParseFile {
        error: syn::Error,
        filepath: PathBuf,
        source_code: String,
    },
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IncorrectUsage => write!(
                f,
                "Usage: dump-syntax path/to/source/directory path/to/output/directory"
            ),
            Error::ReadDirectory(error) => write!(f, "Unable to read directory: {}", error),
            Error::ReadFile(error) => write!(f, "Unable to read file: {}", error),
            Error::WriteFile(error) => write!(f, "Unable to write file: {}", error),
            Error::ParseFile {
                error,
                filepath,
                source_code,
            } => render_location(f, error, filepath, source_code),
        }
    }
}

fn main() {
    if let Err(error) = try_main() {
        let _ = writeln!(io::stderr(), "{}", error);
        process::exit(1);
    }
}

fn try_main() -> Result<(), Error> {
    let mut args = env::args_os();
    let _ = args.next(); // executable name

    let (source_directory, output_directory) = match (args.next(), args.next(), args.next()) {
        (Some(source), Some(output), None) => (PathBuf::from(source), PathBuf::from(output)),
        _ => return Err(Error::IncorrectUsage),
    };

    for source_file in rust_files_below(&source_directory).map_err(Error::ReadDirectory)? {
        let syntax = parse(&source_file)?;

        let relative_path = source_file
            .strip_prefix(&source_directory)
            .expect("a file found below the source directory should be relative to it");
        let mut output_file = output_directory.join(relative_path).into_os_string();
        output_file.push(".ast");
        let output_file = PathBuf::from(output_file);

        if let Some(parent) = output_file.parent() {
            fs::create_dir_all(parent).map_err(Error::WriteFile)?;
        }
        fs::write(&output_file, format!("{:#?}\n", syntax)).map_err(Error::WriteFile)?;
    }

    Ok(())
}

fn rust_files_below(directory: &Path) -> io::Result<Vec<PathBuf>> {
    let mut rust_files = Vec::new();
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            rust_files.extend(rust_files_below(&path)?);
        } else if path.extension() == Some(OsStr::new("rs")) {
            rust_files.push(path);
        }
    }
    rust_files.sort();
    Ok(rust_files)
}

fn parse(filepath: &Path) -> Result<syn::File, Error> {
    let code = fs::read_to_string(filepath).map_err(Error::ReadFile)?;
    syn::parse_file(&code).map_err(|error| Error::ParseFile {
        error,
        filepath: filepath.to_path_buf(),
        source_code: code,
    })
}

// Render a rustc-style error message, including colors.
//
//     error: Syn unable to parse file
//       --> main.rs:40:17
//        |
//     40 |     fn fmt(&self formatter: &mut fmt::Formatter) -> fmt::Result {
//        |                  ^^^^^^^^^ expected `,`
//
fn render_location(
    formatter: &mut fmt::Formatter,
    err: &syn::Error,
    filepath: &Path,
    code: &str,
) -> fmt::Result {
    let start = err.span().start();
    let mut end = err.span().end();

    let code_line = match start.line.checked_sub(1).and_then(|n| code.lines().nth(n)) {
        Some(line) => line,
        None => return render_fallback(formatter, err),
    };

    if end.line > start.line {
        end.line = start.line;
        end.column = code_line.len();
    }

    let filename = filepath
        .file_name()
        .map(OsStr::to_string_lossy)
        .unwrap_or(Cow::Borrowed("main.rs"));

    write!(
        formatter,
        "\n\
         {error}{header}\n\
         {indent}{arrow} {filename}:{linenum}:{colnum}\n\
         {indent} {pipe}\n\
         {label} {pipe} {code}\n\
         {indent} {pipe} {offset}{underline} {message}\n\
         ",
        error = "error".red().bold(),
        header = ": Syn unable to parse file".bold(),
        indent = " ".repeat(start.line.to_string().len()),
        arrow = "-->".blue().bold(),
        filename = filename,
        linenum = start.line,
        colnum = start.column,
        pipe = "|".blue().bold(),
        label = start.line.to_string().blue().bold(),
        code = code_line.trim_end(),
        offset = " ".repeat(start.column),
        underline = "^"
            .repeat(end.column.saturating_sub(start.column).max(1))
            .red()
            .bold(),
        message = err.to_string().red(),
    )
}

fn render_fallback(formatter: &mut fmt::Formatter, err: &syn::Error) -> fmt::Result {
    write!(formatter, "Unable to parse file: {}", err)
}
