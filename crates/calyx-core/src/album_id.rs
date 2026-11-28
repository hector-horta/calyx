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
#[path = "../tests/unit/album_id.rs"]
mod tests;
