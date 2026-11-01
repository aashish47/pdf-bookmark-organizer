use anyhow::{bail, Context, Result};
use clap::Parser;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::{
    classification,
    cli::Args,
    diagnostics,
    hierarchy,
    pdf,
};

pub fn run() -> Result<()> {
    let args = Args::parse();
    let input_path = args.input;

    if !input_path.is_file() {
        bail!("File not found: {}", input_path.display());
    }

    let output_path = match args.output {
        Some(path) => path,
        None => default_output_path(&input_path)?,
    };

    println!("Input : {}", input_path.display());

    let document = pdf::open_pdf(&input_path)?;
    let (mut bookmarks, toc_errors) = pdf::read_bookmarks(&document)?;

    if bookmarks.is_empty() {
        bail!("PDF has no readable existing bookmarks.");
    }

    println!("\nExisting bookmarks: {}", bookmarks.len());
    if !toc_errors.is_empty() {
        eprintln!("\nWarning: lopdf reported {} outline decoding issue(s):", toc_errors.len());
        for error in &toc_errors {
            eprintln!("  - {error}");
        }
    }

    diagnostics::print_top_level(&bookmarks);

    let (content_start, back_start) = match classification::classify(&mut bookmarks) {
        Ok(boundaries) => boundaries,
        Err(error) => {
            diagnostics::print_existing_bookmarks(&bookmarks);
            return Err(error);
        }
    };

    let content_title = &bookmarks
        .iter()
        .find(|bookmark| bookmark.index == content_start)
        .context("Internal error: Content boundary index was not found")?
        .title;

    println!(
        "\nContent begins at bookmark index {}: {}",
        content_start, content_title
    );

    match back_start {
        Some(index) => {
            let title = &bookmarks
                .iter()
                .find(|bookmark| bookmark.index == index)
                .context("Internal error: Back Matter boundary index was not found")?
                .title;
            println!("Back Matter begins at bookmark index {index}: {title}");
        }
        None => println!("Back Matter marker was not explicitly detected."),
    }

    diagnostics::print_existing_bookmarks(&bookmarks);

    let new_toc = hierarchy::build_new_toc(&bookmarks)
        .context("Could not construct a valid replacement bookmark hierarchy")?;
    diagnostics::print_proposed_toc(&new_toc);

    if args.dry_run {
        println!("\nDRY RUN: PDF was NOT modified.");
        return Ok(());
    }

    if paths_refer_to_same_file(&input_path, &output_path) {
        bail!(
            "Input and output paths refer to the same file. Choose a different output path."
        );
    }

    pdf::write_bookmarks(document, &output_path, &new_toc)?;

    println!("\n{}\nSUCCESS\n{}", "=".repeat(78), "=".repeat(78));
    println!("Output: {}", output_path.display());
    Ok(())
}

fn default_output_path(input: &Path) -> Result<PathBuf> {
    let parent = input.parent().unwrap_or_else(|| Path::new("."));
    let stem = input
        .file_stem()
        .context("Input path does not have a valid file name")?;
    let extension = input.extension();

    let mut filename = OsString::from(stem);
    filename.push(" - Revised Bookmarks");
    if let Some(extension) = extension {
        filename.push(".");
        filename.push(extension);
    }

    Ok(parent.join(filename))
}

fn paths_refer_to_same_file(input: &Path, output: &Path) -> bool {
    if input == output {
        return true;
    }

    match (input.canonicalize(), output.canonicalize()) {
        (Ok(input), Ok(output)) => input == output,
        _ => false,
    }
}
