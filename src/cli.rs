use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "pdf-bookmark-organizer",
    version,
    about = "Reorganize PDF bookmarks into Front Matter / Content / Back Matter"
)]
pub struct Args {
    /// Input PDF file
    pub input: PathBuf,

    /// Output PDF file (defaults to '<input> - Revised Bookmarks.pdf')
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Print the proposed outline without writing a PDF
    #[arg(long)]
    pub dry_run: bool,
}
