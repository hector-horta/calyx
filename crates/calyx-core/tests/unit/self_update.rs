use super::*;
use flate2::write::GzEncoder;
use std::io::Write;
use tar::{Builder, Header};
use zip::ZipWriter;

fn zip_with(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    for (name, data) in entries {
        writer
            .start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(data).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

fn tar_gz_with(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let gz = GzEncoder::new(Vec::new(), flate2::Compression::default());
    let mut builder = Builder::new(gz);
    for (name, data) in entries {
        let mut header = Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();
        builder.append_data(&mut header, name, *data).unwrap();
    }
    builder.into_inner().unwrap().finish().unwrap()
}

#[test]
fn picks_the_checksum_of_the_asset() {
    let a = "a".repeat(64);
    let b = "B".repeat(64);
    let sums = format!("{a}  other.zip\n{b} *calyx.zip\n");
    assert_eq!(expected_sha256(&sums, "calyx.zip"), Some("b".repeat(64)));
    assert_eq!(expected_sha256(&sums, "missing.zip"), None);
    assert_eq!(expected_sha256("xyz  calyx.zip", "calyx.zip"), None);
}

#[test]
fn hashes_as_lowercase_hex() {
    assert_eq!(
        sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn urls_are_rebuilt_from_the_version() {
    assert_eq!(
        asset_url("v1.2.3", "x.zip"),
        Some("https://github.com/hector-horta/calyx/releases/download/v1.2.3/x.zip".to_string())
    );
    assert_eq!(asset_url("../evil", "x.zip"), None);
}

#[test]
fn only_the_root_binary_is_extracted() {
    let zip = zip_with(&[
        ("README.md", b"doc"),
        ("assets/calyx.exe", b"nested"),
        (BIN_NAME, b"bin"),
    ]);
    assert_eq!(binary_from_zip(&zip).unwrap(), b"bin");

    let tgz = tar_gz_with(&[
        ("./README.md", b"doc"),
        ("./assets/calyx", b"nested"),
        (&format!("./{BIN_NAME}"), b"bin"),
    ]);
    assert_eq!(binary_from_tar_gz(&tgz).unwrap(), b"bin");
}

#[test]
fn a_missing_binary_is_an_error() {
    let zip = zip_with(&[("README.md", b"doc")]);
    assert!(binary_from_zip(&zip).is_err());

    let tgz = tar_gz_with(&[("README.md", b"doc")]);
    assert!(binary_from_tar_gz(&tgz).is_err());
}

#[test]
fn package_manager_installs_are_left_alone() {
    assert!(is_managed(Path::new(
        "C:\\Users\\me\\.cargo\\bin\\calyx.exe"
    )));
    assert!(is_managed(Path::new(
        "/opt/homebrew/Cellar/calyx/bin/calyx"
    )));
    assert!(!is_managed(Path::new("C:\\Tools\\calyx\\calyx.exe")));
}

#[test]
fn the_running_binary_is_swapped_for_the_new_one() {
    let dir = std::env::temp_dir().join(format!("calyx-self-update-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let exe = dir.join(BIN_NAME);
    std::fs::write(&exe, b"old").unwrap();

    replace_exe(&exe, b"new").unwrap();
    assert_eq!(std::fs::read(&exe).unwrap(), b"new");
    assert_eq!(std::fs::read(old_path(&exe)).unwrap(), b"old");
    assert!(!exe.with_extension("new").exists());

    std::fs::remove_dir_all(&dir).unwrap();
}
