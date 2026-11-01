use anyhow::{bail, Context, Result};
use lopdf::{Bookmark as PdfBookmark, Document, Object};
use std::collections::BTreeMap;
use std::path::Path;

use crate::model::{Bookmark, OutlineEntry};

pub fn open_pdf(path: &Path) -> Result<Document> {
    Document::load(path).with_context(|| format!("Could not open PDF: {}", path.display()))
}

/// Read lopdf's flat TOC and convert it into the application's model.
/// lopdf reports page numbers as 1-based values, matching the original script.
pub fn read_bookmarks(document: &Document) -> Result<(Vec<Bookmark>, Vec<String>)> {
    let toc = document
        .get_toc()
        .context("Could not read the PDF outline. The PDF may not contain usable bookmarks.")?;

    let mut bookmarks = Vec::with_capacity(toc.toc.len());
    for (index, entry) in toc.toc.into_iter().enumerate() {
        let page = u32::try_from(entry.page)
            .with_context(|| format!("Bookmark '{}' has an invalid page number", entry.title))?;
        bookmarks.push(Bookmark {
            index,
            level: entry.level,
            title: crate::model::normalize_title(&entry.title),
            page,
            section: None,
        });
    }

    Ok((bookmarks, toc.errors))
}

/// Replace the source outline with the proposed outline.
/// The PDF pages themselves are not edited.
pub fn write_bookmarks(
    mut document: Document,
    output_path: &Path,
    toc: &[OutlineEntry],
) -> Result<()> {
    let page_ids: BTreeMap<u32, lopdf::ObjectId> = document.get_pages();

    // Remove the old outline objects before installing the replacement tree.
    document
        .delete_outlines()
        .context("Could not remove the existing PDF outline")?;

    // lopdf represents bookmark destinations using page-object IDs, not page numbers.
    // Keep a stack of the most recent bookmark at each level to reconstruct parents.
    let mut parent_stack: Vec<(usize, u32)> = Vec::new();

    for entry in toc {
        while parent_stack
            .last()
            .is_some_and(|(parent_level, _)| *parent_level >= entry.level)
        {
            parent_stack.pop();
        }

        let parent_id = parent_stack.last().map(|(_, id)| *id);
        if entry.level > 1 && parent_id.is_none() {
            bail!("Cannot attach bookmark '{}' at level {}: parent is missing.", entry.title, entry.level);
        }
        if let Some((parent_level, _)) = parent_stack.last() {
            if *parent_level + 1 != entry.level {
                bail!(
                    "Invalid parent level for '{}': parent L{} then child L{}.",
                    entry.title,
                    parent_level,
                    entry.level
                );
            }
        }

        let page_id = page_ids.get(&entry.page).copied().with_context(|| {
            format!(
                "Bookmark '{}' points to PDF page {}, but the document has no such page.",
                entry.title, entry.page
            )
        })?;

        let bookmark = PdfBookmark::new(entry.title.clone(), [0.0, 0.0, 0.0], 0, page_id);
        let bookmark_id = document.add_bookmark(bookmark, parent_id);
        parent_stack.push((entry.level, bookmark_id));
    }

    let outline_root = document
        .build_outline()
        .context("Could not construct the replacement PDF outline")?;

    document
        .catalog_mut()
        .context("Could not access the PDF catalog")?
        .set("Outlines", Object::Reference(outline_root));

    document
        .save(output_path)
        .with_context(|| format!("Could not save output PDF: {}", output_path.display()))?;

    Ok(())
}
