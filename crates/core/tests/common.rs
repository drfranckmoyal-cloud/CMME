#![allow(dead_code)]
use cmme_core::Store;
use std::path::PathBuf;

pub const KEY: [u8; 32] = [7u8; 32];

pub fn tmpdir() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

pub fn store_in(dir: &tempfile::TempDir) -> (Store, PathBuf) {
    let p = dir.path().join("test.db");
    (Store::open(&p, &KEY, true, "Praticien test").unwrap(), p)
}

pub fn specs_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../specs")
}
