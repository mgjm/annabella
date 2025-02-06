use std::{env, fs, path::PathBuf};

use annabella::{codegen, parser, tokenizer::TokenStream, Error};
use anyhow::{Context, Result};

fn main() -> Result<()> {
    let path: PathBuf = env::args_os().nth(1).context("ada source path")?.into();

    let source = fs::read_to_string(&path).with_context(|| format!("read source: {path:?}"))?;

    run(source, path).map_err(show_error)
}

fn run(source: String, path: PathBuf) -> Result<(), Error> {
    let input = TokenStream::parse(&source, Some(path))?;

    let items = parser::parse(input)?;

    let code = codegen::run(items)?;
    println!("{code}");

    Ok(())
}

fn show_error(err: Error) -> anyhow::Error {
    let Error {
        span,
        msg,
        recoverable: _,
    } = err;
    eprintln!(
        "{msg} in {:?}:",
        span.filepath().as_deref().unwrap_or("<call_site>".as_ref())
    );
    for (line, source, start, end) in span.lines() {
        eprintln!("{line:4}: {source}");
        let mut marker = Vec::new();
        if let Some(start) = start {
            marker.resize(start as usize, b' ');
            marker.push(b'^');
        }
        if let Some(end) = end {
            marker.resize(end as usize + 1, b'-');
        }
        if !marker.is_empty() {
            eprintln!("      {} {msg}", std::str::from_utf8(&marker).unwrap());
        }
    }
    anyhow::anyhow!("transpiler failed")
}
