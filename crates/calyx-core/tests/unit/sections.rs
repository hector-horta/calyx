use super::*;

#[test]
fn new_releases_query_uses_a_plausible_year() {
    let q = SectionType::NewReleases.search_query();
    let year: u64 = q.trim_start_matches("top hits ").parse().unwrap();
    assert!((2024..2100).contains(&year), "{q}");
}
