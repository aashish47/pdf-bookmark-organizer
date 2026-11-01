use crate::model::{Bookmark, OutlineEntry};

pub fn print_top_level(bookmarks: &[Bookmark]) {
    println!("\n{}\nEXISTING TOP-LEVEL BOOKMARKS\n{}", "=".repeat(78), "=".repeat(78));

    for bookmark in bookmarks.iter().filter(|bookmark| bookmark.level == 1) {
        let section = bookmark.section.map_or("UNKNOWN", |s| s.label());
        println!(
            "{:3}  page={:4}  {:7}  {}",
            bookmark.index, bookmark.page, section, bookmark.title
        );
    }
}

pub fn print_existing_bookmarks(bookmarks: &[Bookmark]) {
    println!("\n{}\nEXISTING BOOKMARKS + CLASSIFICATION\n{}", "=".repeat(78), "=".repeat(78));

    for bookmark in bookmarks {
        let indent = "    ".repeat(bookmark.level.saturating_sub(1));
        let section = bookmark.section.map_or("UNKNOWN", |s| s.label());
        println!(
            "{:4}  page={:5}  L{}  {:7}  {}{}",
            bookmark.index,
            bookmark.page,
            bookmark.level,
            section,
            indent,
            bookmark.title
        );
    }
}

pub fn print_proposed_toc(toc: &[OutlineEntry]) {
    println!("\n{}\nPROPOSED BOOKMARK STRUCTURE\n{}", "=".repeat(78), "=".repeat(78));

    for entry in toc {
        let indent = "    ".repeat(entry.level.saturating_sub(1));
        if entry.level == 1 {
            println!("[STRUCTURAL] {} (PDF page {})", entry.title, entry.page);
        } else {
            println!("{}{} (PDF page {})", indent, entry.title, entry.page);
        }
    }
}
