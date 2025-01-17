use std::{env, fs, path::PathBuf};

use annabella_rs::{
    codegen,
    parser::{self, Stmt},
    tokenizer::{Span, TokenStream},
};
use anyhow::{Context, Result};

fn main() -> Result<()> {
    let path: PathBuf = env::args_os().nth(1).context("ada source path")?.into();

    let source = fs::read_to_string(&path).with_context(|| format!("read source: {path:?}"))?;

    let input = match TokenStream::parse(&source, Some(path)) {
        Ok(stream) => stream,
        Err(annabella_rs::tokenizer::Error::InvalidSyntax(span, msg)) => show_error(span, &msg)?,
    };

    let stmts: Vec<Stmt> = match parser::parse(input) {
        Ok(stmt) => stmt,
        Err(parser::Error { span, msg, .. }) => show_error(span, &msg)?,
    };

    let code = match codegen::run(stmts) {
        Ok(code) => code,
        Err(parser::Error { span, msg, .. }) => show_error(span, &msg)?,
    };
    println!("{code}");

    Ok(())
}

fn show_error<T>(span: Span, msg: &str) -> Result<T> {
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
    anyhow::bail!("transpiler failed");
}
