// Copyright 2013-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![deny(rust_2018_idioms)]

use std::{ffi::OsStr, fs};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::thread;

use tempfile::{Builder, TempDir};

macro_rules! transmute {
    ($code:expr) => {
        unsafe { std::mem::transmute::<_, &Path>($code) }
    };
}

fn test_tempdir() {
    let path = {
        let p = Builder::new().prefix("foobar").tempdir_in(transmute!("."), |p1, p2| transmute!(p1).join(transmute!(p2))).unwrap();
        let p = p.path();
        assert!(p.to_str().unwrap().contains("foobar"));
        p.to_path_buf()
    };
    assert!(!path.exists());
}

fn test_prefix() {
    let tmpfile = TempDir::with_prefix_in("prefix", transmute!("."), |p1, p2| transmute!(p1).join(transmute!(p2))).unwrap();
    let name = tmpfile.path().file_name().unwrap().to_str().unwrap();
    assert!(name.starts_with("prefix"));
}

fn test_suffix() {
    let tmpfile = TempDir::with_suffix_in("suffix", transmute!("."), |p1, p2| transmute!(p1).join(transmute!(p2))).unwrap();
    let name = tmpfile.path().file_name().unwrap().to_str().unwrap();
    assert!(name.ends_with("suffix"));
}

fn test_customnamed() {
    let tmpfile = Builder::new()
        .prefix("prefix")
        .suffix("suffix")
        .rand_bytes(12)
        .tempdir(|p1, p2| transmute!(p1).join(transmute!(p2)))
        .unwrap();
    let name = tmpfile.path().file_name().unwrap().to_str().unwrap();
    assert!(name.starts_with("prefix"));
    assert!(name.ends_with("suffix"));
    assert_eq!(name.len(), 24);
}

fn test_rm_tempdir() {
    let (tx, rx) = channel();
    let f = move || {
        let tmp = TempDir::new(|p1, p2| transmute!(p1).join(transmute!(p2))).unwrap();
        tx.send(tmp.path().to_path_buf()).unwrap();
        panic!("panic to unwind past `tmp`");
    };
    let _ = thread::spawn(f).join();
    let path = rx.recv().unwrap();
    assert!(!path.exists());

    let tmp = TempDir::new(|p1, p2| transmute!(p1).join(transmute!(p2))).unwrap();
    let path = tmp.path().to_path_buf();
    let f = move || {
        let _tmp = tmp;
        panic!("panic to unwind past `tmp`");
    };
    let _ = thread::spawn(f).join();
    assert!(!path.exists());

    let path;
    {
        let f = move || TempDir::new(|p1, p2| transmute!(p1).join(transmute!(p2))).unwrap();

        let tmp = thread::spawn(f).join().unwrap();
        path = tmp.path().to_path_buf();
        assert!(path.exists());
    }
    assert!(!path.exists());

    let path;
    {
        let tmp = TempDir::new(|p1, p2| transmute!(p1).join(transmute!(p2))).unwrap();
        path = tmp.into_path();
    }
    assert!(path.exists());
    fs::remove_dir_all(&path).unwrap();
    assert!(!path.exists());
}

fn test_rm_tempdir_close() {
    let (tx, rx) = channel();
    let f = move || {
        let tmp = TempDir::new(|p1, p2| transmute!(p1).join(transmute!(p2))).unwrap();
        tx.send(tmp.path().to_path_buf()).unwrap();
        tmp.close().unwrap();
        panic!("panic when unwinding past `tmp`");
    };
    let _ = thread::spawn(f).join();
    let path = rx.recv().unwrap();
    assert!(!path.exists());

    let tmp = TempDir::new(|p1, p2| transmute!(p1).join(transmute!(p2))).unwrap();
    let path = tmp.path().to_path_buf();
    let f = move || {
        let tmp = tmp;
        tmp.close().unwrap();
        panic!("panic when unwinding past `tmp`");
    };
    let _ = thread::spawn(f).join();
    assert!(!path.exists());

    let path;
    {
        let f = move || TempDir::new(|p1, p2| transmute!(p1).join(transmute!(p2))).unwrap();

        let tmp = thread::spawn(f).join().unwrap();
        path = tmp.path().to_path_buf();
        assert!(path.exists());
        tmp.close().unwrap();
    }
    assert!(!path.exists());

    let path;
    {
        let tmp = TempDir::new(|p1, p2| transmute!(p1).join(transmute!(p2))).unwrap();
        path = tmp.into_path();
    }
    assert!(path.exists());
    fs::remove_dir_all(&path).unwrap();
    assert!(!path.exists());
}

fn dont_double_panic() {
    let r: Result<(), _> = thread::spawn(move || {
        let tmpdir = TempDir::new(|p1, p2| transmute!(p1).join(transmute!(p2))).unwrap();
        // Remove the temporary directory so that TempDir sees
        // an error on drop
        fs::remove_dir(tmpdir.path()).unwrap();
        // Panic. If TempDir panics *again* due to the rmdir
        // error then the process will abort.
        panic!();
    })
    .join();
    assert!(r.is_err());
}

fn in_tmpdir<F>(f: F)
where
    F: FnOnce(),
{
    let tmpdir = TempDir::new(|p1, p2| transmute!(p1).join(transmute!(p2))).unwrap();
    assert!(std::env::set_current_dir(tmpdir.path()).is_ok());

    f();
}

fn pass_as_asref_path() {
    let tempdir = TempDir::new(|p1, p2| transmute!(p1).join(transmute!(p2))).unwrap();
    takes_asref_path(&tempdir);

    fn takes_asref_path<T: AsRef<Path>>(path: T) {
        let path = path.as_ref();
        assert!(path.exists());
    }
}

fn test_keep() {
    let tmpdir = Builder::new().keep(true).tempdir(|p1, p2| transmute!(p1).join(transmute!(p2))).unwrap();
    let path = tmpdir.path().to_owned();
    drop(tmpdir);
    assert!(path.exists());
    fs::remove_dir(path).unwrap();
}

#[test]
fn main() {
    in_tmpdir(test_tempdir);
    in_tmpdir(test_prefix);
    in_tmpdir(test_suffix);
    in_tmpdir(test_customnamed);
    in_tmpdir(test_rm_tempdir);
    in_tmpdir(test_rm_tempdir_close);
    in_tmpdir(dont_double_panic);
    in_tmpdir(pass_as_asref_path);
    in_tmpdir(test_keep);
}
