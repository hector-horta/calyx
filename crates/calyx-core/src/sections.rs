//! Catalog searches behind the curated sections (Home, New releases, Radio).

use crate::backend::SectionType;

/// "top hits <current year>" without pulling in a date crate.
fn new_releases_query() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("top hits {}", 1970 + secs / 31_556_952)
}

impl SectionType {
    /// Catalog search term that fills this section.
    pub(crate) fn search_query(self) -> String {
        match self {
            SectionType::Home => "top hits".to_string(),
            SectionType::NewReleases => new_releases_query(),
            SectionType::Radio => "electronic chill radio mix".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_releases_query_uses_a_plausible_year() {
        let q = SectionType::NewReleases.search_query();
        let year: u64 = q.trim_start_matches("top hits ").parse().unwrap();
        assert!((2024..2100).contains(&year), "{q}");
    }
}
