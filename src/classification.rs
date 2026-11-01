use anyhow::{bail, Result};
use regex::Regex;
use std::sync::LazyLock;

use crate::model::{normalized_lower, Bookmark, Section};

const FRONT_TITLES: &[&str] = &[
    "cover",
    "half title",
    "title",
    "title page",
    "copyright",
    "copyright page",
    "dedication",
    "contents",
    "table of contents",
    "toc",
    "preface",
    "foreword",
    "about the author",
    "about the authors",
    "acknowledgments",
    "acknowledgements",
    "acronyms",
    "list of figures",
    "list of tables",
    "list of illustrations",
    "list of abbreviations",
];

static PART_MARKER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^part\s+(\d+|one|two|three|four|five|six|seven|eight|nine|ten|eleven|twelve|thirteen|fourteen|fifteen|sixteen|seventeen|eighteen|nineteen|twenty|[ivxlcdm]+)\b",
    )
    .expect("valid part-marker regex")
});

static CHAPTER_MARKER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(chapter\s+\d+\b|ch\.?\s+\d+\b|\d+\s+\S+|\d+\.\s+\S+)")
        .expect("valid chapter-marker regex")
});

static ROMAN_PART_MARKER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[ivxlcdm]+\s+.+").expect("valid Roman-part regex")
});

static BACK_MARKER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(bibliography|selected bibliography|glossary|index|subject index|author index|name index|general index|credits|photo credits|illustration credits|image credits|eula)\b",
    )
    .expect("valid back-matter regex")
});

pub fn is_front_title(title: &str) -> bool {
    let title = normalized_lower(title);
    FRONT_TITLES.contains(&title.as_str())
        || title.starts_with("contents")
        || title.starts_with("preface")
        || title.starts_with("acknowledg")
}

/// This predicate is only applied to level-1 bookmarks.
/// Appendix, References, Further Reading, Answers, and Solutions are
/// deliberately not global back-matter boundaries.
pub fn is_back_title(title: &str) -> bool {
    BACK_MARKER.is_match(&normalized_lower(title))
}

pub fn is_part_marker(title: &str) -> bool {
    PART_MARKER.is_match(&normalized_lower(title))
}

pub fn is_chapter_marker(title: &str) -> bool {
    CHAPTER_MARKER.is_match(&normalized_lower(title))
}

pub fn is_roman_part_marker(title: &str) -> bool {
    ROMAN_PART_MARKER.is_match(&normalized_lower(title))
}

pub fn is_content_marker(title: &str) -> bool {
    is_part_marker(title) || is_chapter_marker(title) || is_roman_part_marker(title)
}

/// Find global boundaries from top-level bookmarks, then let each descendant
/// inherit the classification of its top-level ancestor.
pub fn classify(bookmarks: &mut [Bookmark]) -> Result<(usize, Option<usize>)> {
    if bookmarks.is_empty() {
        bail!("PDF has no bookmarks.");
    }

    let content_start = bookmarks
        .iter()
        .find(|bookmark| bookmark.level == 1 && is_content_marker(&bookmark.title))
        .map(|bookmark| bookmark.index)
        .ok_or_else(|| anyhow::anyhow!(
            "Could not determine where Content begins. No top-level Part/Chapter marker or recognizable numbered chapter sequence was found."
        ))?;

    let back_start = bookmarks
        .iter()
        .find(|bookmark| {
            bookmark.level == 1
                && bookmark.index > content_start
                && is_back_title(&bookmark.title)
        })
        .map(|bookmark| bookmark.index);

    let mut current_section: Option<Section> = None;

    for bookmark in bookmarks {
        if bookmark.level == 1 {
            current_section = Some(if bookmark.index < content_start {
                Section::Front
            } else if back_start.is_some_and(|start| bookmark.index >= start) {
                Section::Back
            } else {
                Section::Content
            });
        }

        let section = current_section.ok_or_else(|| anyhow::anyhow!(
            "Bookmark '{}' (index {}) is nested under no top-level bookmark.",
            bookmark.title,
            bookmark.index
        ))?;
        bookmark.section = Some(section);
    }

    Ok((content_start, back_start))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chapter_zero_is_a_content_marker() {
        assert!(is_chapter_marker("Ch 0: Introduction"));
    }

    #[test]
    fn numbered_chapters_are_detected() {
        assert!(is_chapter_marker("1 INTRODUCTION"));
        assert!(is_chapter_marker("1. Introduction"));
    }

    #[test]
    fn appendix_is_not_a_global_back_marker() {
        assert!(!is_back_title("Appendix A"));
    }

    #[test]
    fn chapter_end_material_is_not_a_global_back_marker() {
        assert!(!is_back_title("References"));
        assert!(!is_back_title("Further Reading"));
        assert!(!is_back_title("Answers"));
        assert!(!is_back_title("Solutions"));
    }

    #[test]
    fn roman_numeral_section_is_detected() {
        assert!(is_roman_part_marker("VIII Appendix: Mathematical Background"));
    }

    #[test]
    fn back_pattern_requires_a_word_boundary() {
        assert!(is_back_title("Index of terms"));
        assert!(!is_back_title("Indexing techniques"));
    }

    #[test]
    fn first_top_level_content_marker_wins_and_children_inherit() {
        let mut items = vec![
            item(0, 1, "Cover"),
            item(1, 1, "Ch 0: Introduction"),
            item(2, 2, "References"),
            item(3, 1, "Part 1: Foundations"),
            item(4, 1, "Selected Bibliography"),
            item(5, 2, "A"),
        ];

        let (content, back) = classify(&mut items).unwrap();
        assert_eq!(content, 1);
        assert_eq!(back, Some(4));
        assert_eq!(items[0].section, Some(Section::Front));
        assert_eq!(items[2].section, Some(Section::Content));
        assert_eq!(items[5].section, Some(Section::Back));
    }

    fn item(index: usize, level: usize, title: &str) -> Bookmark {
        Bookmark {
            index,
            level,
            title: title.to_string(),
            page: index as u32 + 1,
            section: None,
        }
    }
}
