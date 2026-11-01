use anyhow::{bail, Result};
use regex::Regex;

use crate::model::{normalized_lower, Bookmark, OutlineEntry, Section};

/// Apply the original Python script's appendix-letter correction.
pub fn fix_special_hierarchies(entries: &mut [OutlineEntry]) {
    let appendix = Regex::new(r"^appendix\b").expect("valid appendix regex");
    let lettered_section = Regex::new(r"^[a-z]\s+").expect("valid appendix-letter regex");
    let mut appendix_parent_level: Option<usize> = None;

    for entry in entries {
        let title = normalized_lower(&entry.title);

        if appendix.is_match(&title) {
            appendix_parent_level = Some(entry.level);
        } else if let Some(parent_level) = appendix_parent_level {
            if lettered_section.is_match(&title) {
                if entry.level <= parent_level {
                    entry.level = parent_level + 1;
                }
            } else if entry.level <= parent_level {
                appendix_parent_level = None;
            }
        }
    }
}

pub fn validate_toc(toc: &[OutlineEntry]) -> Result<()> {
    if toc.is_empty() {
        bail!("Generated bookmark hierarchy is empty.");
    }

    let mut previous_level = 0usize;
    for (index, entry) in toc.iter().enumerate() {
        if entry.level < 1 {
            bail!("Invalid hierarchy level 0 for bookmark '{}'.", entry.title);
        }
        if index == 0 && entry.level != 1 {
            bail!("First bookmark '{}' must be level 1.", entry.title);
        }
        if index > 0 && entry.level > previous_level + 1 {
            bail!(
                "Invalid hierarchy jump before '{}': L{} -> L{}",
                entry.title,
                previous_level,
                entry.level
            );
        }
        previous_level = entry.level;
    }

    Ok(())
}

/// Construct the new flattened TOC, preserving source levels beneath each
/// added structural heading.
pub fn build_new_toc(bookmarks: &[Bookmark]) -> Result<Vec<OutlineEntry>> {
    let mut toc = Vec::new();

    for section in [Section::Front, Section::Content, Section::Back] {
        let group: Vec<&Bookmark> = bookmarks
            .iter()
            .filter(|bookmark| bookmark.section == Some(section))
            .collect();

        if group.is_empty() {
            continue;
        }

        toc.push(OutlineEntry {
            level: 1,
            title: section.structural_title().to_string(),
            page: group[0].page,
        });

        for bookmark in group {
            toc.push(OutlineEntry {
                level: bookmark.level + 1,
                title: bookmark.title.clone(),
                page: bookmark.page,
            });
        }
    }

    fix_special_hierarchies(&mut toc);
    validate_toc(&toc)?;
    Ok(toc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_bookmarks_and_shifts_original_levels() {
        let bookmarks = vec![
            item(0, 1, "Cover", 1, Section::Front),
            item(1, 1, "Part 1", 5, Section::Content),
            item(2, 2, "Chapter 1", 6, Section::Content),
            item(3, 3, "A subsection", 7, Section::Content),
            item(4, 1, "Index", 90, Section::Back),
        ];

        let toc = build_new_toc(&bookmarks).unwrap();
        assert_eq!(toc[0], OutlineEntry { level: 1, title: "Front Matter".into(), page: 1 });
        assert_eq!(toc[1].level, 2);
        assert_eq!(toc[4].level, 3);
        assert_eq!(toc[5].level, 4);
        assert_eq!(toc[6].title, "Back Matter");
        assert_eq!(toc[7].title, "Index");
    }

    #[test]
    fn lettered_appendix_entries_become_children_of_appendix() {
        let mut toc = vec![
            OutlineEntry { level: 1, title: "Content".into(), page: 1 },
            OutlineEntry { level: 2, title: "Appendix: Math".into(), page: 10 },
            OutlineEntry { level: 2, title: "A Sets".into(), page: 11 },
            OutlineEntry { level: 2, title: "B Matrices".into(), page: 12 },
            OutlineEntry { level: 2, title: "Chapter 9".into(), page: 20 },
        ];

        fix_special_hierarchies(&mut toc);
        assert_eq!(toc[2].level, 3);
        assert_eq!(toc[3].level, 3);
        assert_eq!(toc[4].level, 2);
    }

    #[test]
    fn rejects_illegal_level_jump() {
        let toc = vec![
            OutlineEntry { level: 1, title: "Root".into(), page: 1 },
            OutlineEntry { level: 3, title: "Skipped parent".into(), page: 2 },
        ];
        assert!(validate_toc(&toc).is_err());
    }

    fn item(index: usize, level: usize, title: &str, page: u32, section: Section) -> Bookmark {
        Bookmark {
            index,
            level,
            title: title.to_string(),
            page,
            section: Some(section),
        }
    }
}
