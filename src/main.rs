fn main() {
    if let Err(error) = pdf_bookmark_organizer::app::run() {
        eprintln!("\nERROR: {error:#}");
        std::process::exit(1);
    }
}
