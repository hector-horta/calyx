//! Catalog album id of a song, read from Apple Music song JSON.

use serde_json::Value;

pub(crate) fn is_catalog_album_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 20 && id.chars().all(|c| c.is_ascii_digit())
}

pub(crate) fn album_id_from_url(url: &str) -> Option<String> {
    let after = url.split_once("/album/")?.1;
    let path = after.split(['?', '#']).next()?;
    let last = path.split('/').rfind(|s| !s.is_empty())?;
    is_catalog_album_id(last).then(|| last.to_string())
}

pub(crate) fn album_id_from_song(item: &Value) -> Option<String> {
    const ID_POINTERS: [&str; 2] = [
        "/relationships/albums/data/0/id",
        "/relationships/catalog/data/0/relationships/albums/data/0/id",
    ];
    const URL_POINTERS: [&str; 2] = [
        "/attributes/url",
        "/relationships/catalog/data/0/attributes/url",
    ];
    let str_at = |ptr: &str| item.pointer(ptr).and_then(Value::as_str);
    ID_POINTERS
        .iter()
        .filter_map(|ptr| str_at(ptr))
        .find(|id| is_catalog_album_id(id))
        .map(str::to_string)
        .or_else(|| {
            URL_POINTERS
                .iter()
                .filter_map(|ptr| str_at(ptr))
                .find_map(album_id_from_url)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn url_with_slug_and_song_param() {
        assert_eq!(
            album_id_from_url(
                "https://music.apple.com/us/album/lets-dance/1440857781?i=1440857795"
            ),
            Some("1440857781".to_string())
        );
    }

    #[test]
    fn url_without_slug() {
        assert_eq!(
            album_id_from_url("https://music.apple.com/us/album/1440857781"),
            Some("1440857781".to_string())
        );
    }

    #[test]
    fn url_rejects_non_album_and_bad_ids() {
        assert_eq!(
            album_id_from_url("https://music.apple.com/us/song/x/123"),
            None
        );
        assert_eq!(
            album_id_from_url("https://music.apple.com/us/album/x/abc"),
            None
        );
    }

    #[test]
    fn relationship_wins_over_url() {
        let item = json!({
            "relationships": {"albums": {"data": [{"id": "111"}]}},
            "attributes": {"url": "https://music.apple.com/us/album/x/222"}
        });
        assert_eq!(album_id_from_song(&item), Some("111".to_string()));
    }

    #[test]
    fn falls_back_to_url() {
        let item = json!({
            "attributes": {"url": "https://music.apple.com/us/album/x/222?i=3"}
        });
        assert_eq!(album_id_from_song(&item), Some("222".to_string()));
    }

    #[test]
    fn library_song_uses_catalog_relationship() {
        let item = json!({
            "relationships": {"catalog": {"data": [{"attributes": {"url": "https://music.apple.com/us/album/x/333?i=4"}}]}}
        });
        assert_eq!(album_id_from_song(&item), Some("333".to_string()));
    }

    #[test]
    fn missing_everything() {
        let item = json!({"id": "i.abc"});
        assert_eq!(album_id_from_song(&item), None);
    }
}
