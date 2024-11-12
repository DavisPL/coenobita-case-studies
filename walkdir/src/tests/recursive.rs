use std::fs;
use std::path::{Path, PathBuf};

use crate::tests::util::Dir;
use crate::WalkDir;

#[test]
fn send_sync_traits() {
    use crate::{FilterEntry, IntoIter};

    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<WalkDir>();
    assert_sync::<WalkDir>();
    assert_send::<IntoIter>();
    assert_sync::<IntoIter>();
    assert_send::<FilterEntry<IntoIter, u8>>();
    assert_sync::<FilterEntry<IntoIter, u8>>();
}

#[test]
fn empty() {
    let dir = Dir::tmp();
    let wd = WalkDir::new(dir.path());
    let r = dir.run_recursive(wd);
    r.assert_no_errors();

    assert_eq!(1, r.ents().len());
    let ent = &r.ents()[0];
    assert!(ent.file_type().is_dir());
    assert!(!ent.path_is_symlink());
    assert_eq!(0, ent.depth());
    assert_eq!(dir.path(), ent.path());
    assert_eq!(dir.path().file_name().unwrap(), ent.file_name());
}

#[test]
fn empty_follow() {
    let dir = Dir::tmp();
    let wd = WalkDir::new(dir.path()).follow_links(true);
    let r = dir.run_recursive(wd);
    r.assert_no_errors();

    assert_eq!(1, r.ents().len());
    let ent = &r.ents()[0];
    assert!(ent.file_type().is_dir());
    assert!(!ent.path_is_symlink());
    assert_eq!(0, ent.depth());
    assert_eq!(dir.path(), ent.path());
    assert_eq!(dir.path().file_name().unwrap(), ent.file_name());
}

#[test]
fn empty_file() {
    let dir = Dir::tmp();
    dir.touch(Path::new("a"));

    let wd = WalkDir::new(dir.path().join(Path::new("a")));
    let r = dir.run_recursive(wd);
    r.assert_no_errors();

    assert_eq!(1, r.ents().len());
    let ent = &r.ents()[0];
    assert!(ent.file_type().is_file());
    assert!(!ent.path_is_symlink());
    assert_eq!(0, ent.depth());
    assert_eq!(dir.join(Path::new("a")), ent.path());
    assert_eq!("a", ent.file_name());
}

#[test]
fn empty_file_follow() {
    let dir = Dir::tmp();
    dir.touch(Path::new("a"));

    let wd = WalkDir::new(dir.path().join(Path::new("a"))).follow_links(true);
    let r = dir.run_recursive(wd);
    r.assert_no_errors();

    assert_eq!(1, r.ents().len());
    let ent = &r.ents()[0];
    assert!(ent.file_type().is_file());
    assert!(!ent.path_is_symlink());
    assert_eq!(0, ent.depth());
    assert_eq!(dir.join(Path::new("a")), ent.path());
    assert_eq!("a", ent.file_name());
}

#[test]
fn one_dir() {
    let dir = Dir::tmp();
    dir.mkdirp(Path::new("a"));

    let wd = WalkDir::new(dir.path());
    let r = dir.run_recursive(wd);
    r.assert_no_errors();

    let ents = r.ents();
    assert_eq!(2, ents.len());
    let ent = &ents[1];
    assert_eq!(dir.join(Path::new("a")), ent.path());
    assert_eq!(1, ent.depth());
    assert_eq!("a", ent.file_name());
    assert!(ent.file_type().is_dir());
}

#[test]
fn one_file() {
    let dir = Dir::tmp();
    dir.touch(Path::new("a"));

    let wd = WalkDir::new(dir.path());
    let r = dir.run_recursive(wd);
    r.assert_no_errors();

    let ents = r.ents();
    assert_eq!(2, ents.len());
    let ent = &ents[1];
    assert_eq!(dir.join(Path::new("a")), ent.path());
    assert_eq!(1, ent.depth());
    assert_eq!("a", ent.file_name());
    assert!(ent.file_type().is_file());
}

#[test]
fn one_dir_one_file() {
    let dir = Dir::tmp();
    dir.mkdirp(Path::new("foo"));
    dir.touch(Path::new("foo/a"));

    let wd = WalkDir::new(dir.path());
    let r = dir.run_recursive(wd);
    r.assert_no_errors();

    let expected = vec![
        dir.path().to_path_buf(),
        dir.join(Path::new("foo")),
        dir.join(Path::new("foo")).join(Path::new("a")),
    ];
    assert_eq!(expected, r.sorted_paths());
}

#[test]
fn many_files() {
    let dir = Dir::tmp();
    dir.mkdirp(Path::new("foo"));
    dir.touch_all(&[Path::new("foo/a"), Path::new("foo/b"), Path::new("foo/c")]);

    let wd = WalkDir::new(dir.path());
    let r = dir.run_recursive(wd);
    r.assert_no_errors();

    let expected = vec![
        dir.path().to_path_buf(),
        dir.join(Path::new("foo")),
        dir.join(Path::new("foo")).join(Path::new("a")),
        dir.join(Path::new("foo")).join(Path::new("b")),
        dir.join(Path::new("foo")).join(Path::new("c")),
    ];
    assert_eq!(expected, r.sorted_paths());
}

#[test]
fn many_dirs() {
    let dir = Dir::tmp();
    dir.mkdirp(Path::new("foo/a"));
    dir.mkdirp(Path::new("foo/b"));
    dir.mkdirp(Path::new("foo/c"));

    let wd = WalkDir::new(dir.path());
    let r = dir.run_recursive(wd);
    r.assert_no_errors();

    let expected = vec![
        dir.path().to_path_buf(),
        dir.join(Path::new("foo")),
        dir.join(Path::new("foo")).join(Path::new("a")),
        dir.join(Path::new("foo")).join(Path::new("b")),
        dir.join(Path::new("foo")).join(Path::new("c")),
    ];
    assert_eq!(expected, r.sorted_paths());
}

#[test]
fn many_mixed() {
    let dir = Dir::tmp();
    dir.mkdirp(Path::new("foo/a"));
    dir.mkdirp(Path::new("foo/c"));
    dir.mkdirp(Path::new("foo/e"));
    dir.touch_all(&[Path::new("foo/b"), Path::new("foo/d"), Path::new("foo/f")]);

    let wd = WalkDir::new(dir.path());
    let r = dir.run_recursive(wd);
    r.assert_no_errors();

    let expected = vec![
        dir.path().to_path_buf(),
        dir.join(Path::new("foo")),
        dir.join(Path::new("foo")).join(Path::new("a")),
        dir.join(Path::new("foo")).join(Path::new("b")),
        dir.join(Path::new("foo")).join(Path::new("c")),
        dir.join(Path::new("foo")).join(Path::new("d")),
        dir.join(Path::new("foo")).join(Path::new("e")),
        dir.join(Path::new("foo")).join(Path::new("f")),
    ];
    assert_eq!(expected, r.sorted_paths());
}

#[test]
fn nested() {
    let nested =
        PathBuf::from("a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s/t/u/v/w/x/y/z");
    let dir = Dir::tmp();
    dir.mkdirp(&nested);
    dir.touch(nested.join(Path::new("A")));

    let wd = WalkDir::new(dir.path());
    let r = dir.run_recursive(wd);
    r.assert_no_errors();

    let expected = vec![
        dir.path().to_path_buf(),
        dir.join(Path::new("a")),
        dir.join(Path::new("a/b")),
        dir.join(Path::new("a/b/c")),
        dir.join(Path::new("a/b/c/d")),
        dir.join(Path::new("a/b/c/d/e")),
        dir.join(Path::new("a/b/c/d/e/f")),
        dir.join(Path::new("a/b/c/d/e/f/g")),
        dir.join(Path::new("a/b/c/d/e/f/g/h")),
        dir.join(Path::new("a/b/c/d/e/f/g/h/i")),
        dir.join(Path::new("a/b/c/d/e/f/g/h/i/j")),
        dir.join(Path::new("a/b/c/d/e/f/g/h/i/j/k")),
        dir.join(Path::new("a/b/c/d/e/f/g/h/i/j/k/l")),
        dir.join(Path::new("a/b/c/d/e/f/g/h/i/j/k/l/m")),
        dir.join(Path::new("a/b/c/d/e/f/g/h/i/j/k/l/m/n")),
        dir.join(Path::new("a/b/c/d/e/f/g/h/i/j/k/l/m/n/o")),
        dir.join(Path::new("a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p")),
        dir.join(Path::new("a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q")),
        dir.join(Path::new("a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r")),
        dir.join(Path::new("a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s")),
        dir.join(Path::new("a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s/t")),
        dir.join(Path::new("a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s/t/u")),
        dir.join(Path::new("a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s/t/u/v")),
        dir.join(Path::new("a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s/t/u/v/w")),
        dir.join(Path::new("a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s/t/u/v/w/x")),
        dir.join(Path::new("a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s/t/u/v/w/x/y")),
        dir.join(Path::new("a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s/t/u/v/w/x/y/z")),
        dir.join(&nested).join(Path::new("A")),
    ];
    assert_eq!(expected, r.sorted_paths());
}

#[test]
fn nested_small_max_open() {
    let nested =
        PathBuf::from("a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s/t/u/v/w/x/y/z");
    let dir = Dir::tmp();
    dir.mkdirp(&nested);
    dir.touch(nested.join(Path::new("A")));

    let wd = WalkDir::new(dir.path()).max_open(1);
    let r = dir.run_recursive(wd);
    r.assert_no_errors();

    let expected = vec![
        dir.path().to_path_buf(),
        dir.join(Path::new("a")),
        dir.join(Path::new("a/b")),
        dir.join(Path::new("a/b/c")),
        dir.join(Path::new("a/b/c/d")),
        dir.join(Path::new("a/b/c/d/e")),
        dir.join(Path::new("a/b/c/d/e/f")),
        dir.join(Path::new("a/b/c/d/e/f/g")),
        dir.join(Path::new("a/b/c/d/e/f/g/h")),
        dir.join(Path::new("a/b/c/d/e/f/g/h/i")),
        dir.join(Path::new("a/b/c/d/e/f/g/h/i/j")),
        dir.join(Path::new("a/b/c/d/e/f/g/h/i/j/k")),
        dir.join(Path::new("a/b/c/d/e/f/g/h/i/j/k/l")),
        dir.join(Path::new("a/b/c/d/e/f/g/h/i/j/k/l/m")),
        dir.join(Path::new("a/b/c/d/e/f/g/h/i/j/k/l/m/n")),
        dir.join(Path::new("a/b/c/d/e/f/g/h/i/j/k/l/m/n/o")),
        dir.join(Path::new("a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p")),
        dir.join(Path::new("a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q")),
        dir.join(Path::new("a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r")),
        dir.join(Path::new("a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s")),
        dir.join(Path::new("a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s/t")),
        dir.join(Path::new("a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s/t/u")),
        dir.join(Path::new("a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s/t/u/v")),
        dir.join(Path::new("a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s/t/u/v/w")),
        dir.join(Path::new("a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s/t/u/v/w/x")),
        dir.join(Path::new("a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s/t/u/v/w/x/y")),
        dir.join(Path::new("a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s/t/u/v/w/x/y/z")),
        dir.join(&nested).join(Path::new("A")),
    ];
    assert_eq!(expected, r.sorted_paths());
}

#[test]
fn siblings() {
    let dir = Dir::tmp();
    dir.mkdirp(Path::new("foo"));
    dir.mkdirp(Path::new("bar"));
    dir.touch_all(&[Path::new("foo/a"), Path::new("foo/b")]);
    dir.touch_all(&[Path::new("bar/a"), Path::new("bar/b")]);

    let wd = WalkDir::new(dir.path());
    let r = dir.run_recursive(wd);
    r.assert_no_errors();

    let expected = vec![
        dir.path().to_path_buf(),
        dir.join(Path::new("bar")),
        dir.join(Path::new("bar")).join(Path::new("a")),
        dir.join(Path::new("bar")).join(Path::new("b")),
        dir.join(Path::new("foo")),
        dir.join(Path::new("foo")).join(Path::new("a")),
        dir.join(Path::new("foo")).join(Path::new("b")),
    ];
    assert_eq!(expected, r.sorted_paths());
}

#[test]
fn sym_root_file_nofollow() {
    let dir = Dir::tmp();
    dir.touch(Path::new("a"));
    dir.symlink_file(Path::new("a"), Path::new("a-link"));

    let wd = WalkDir::new(dir.join(Path::new("a-link")));
    let r = dir.run_recursive(wd);
    r.assert_no_errors();

    let ents = r.sorted_ents();
    assert_eq!(1, ents.len());
    let link = &ents[0];

    assert_eq!(dir.join(Path::new("a-link")), link.path());

    assert!(link.path_is_symlink());

    assert_eq!(dir.join(Path::new("a")), fs::read_link(link.path()).unwrap());

    assert_eq!(0, link.depth());

    assert!(link.file_type().is_symlink());
    assert!(!link.file_type().is_file());
    assert!(!link.file_type().is_dir());

    assert!(link.metadata().unwrap().file_type().is_symlink());
    assert!(!link.metadata().unwrap().is_file());
    assert!(!link.metadata().unwrap().is_dir());
}

#[test]
fn sym_root_file_follow() {
    let dir = Dir::tmp();
    dir.touch(Path::new("a"));
    dir.symlink_file(Path::new("a"), Path::new("a-link"));

    let wd = WalkDir::new(dir.join(Path::new("a-link"))).follow_links(true);
    let r = dir.run_recursive(wd);
    r.assert_no_errors();

    let ents = r.sorted_ents();
    let link = &ents[0];

    assert_eq!(dir.join(Path::new("a-link")), link.path());

    assert!(link.path_is_symlink());

    assert_eq!(dir.join(Path::new("a")), fs::read_link(link.path()).unwrap());

    assert_eq!(0, link.depth());

    assert!(!link.file_type().is_symlink());
    assert!(link.file_type().is_file());
    assert!(!link.file_type().is_dir());

    assert!(!link.metadata().unwrap().file_type().is_symlink());
    assert!(link.metadata().unwrap().is_file());
    assert!(!link.metadata().unwrap().is_dir());
}

#[test]
fn broken_sym_root_dir_nofollow_and_root_nofollow() {
    let dir = Dir::tmp();
    dir.symlink_dir(Path::new("broken"), Path::new("a-link"));

    let wd = WalkDir::new(dir.join(Path::new("a-link")))
        .follow_links(false)
        .follow_root_links(false);
    let r = dir.run_recursive(wd);
    let ents = r.sorted_ents();
    assert_eq!(ents.len(), 1);
    let link = &ents[0];
    assert_eq!(dir.join(Path::new("a-link")), link.path());
    assert!(link.path_is_symlink());
}

#[test]
fn broken_sym_root_dir_follow_and_root_nofollow() {
    let dir = Dir::tmp();
    dir.symlink_dir(Path::new("broken"), Path::new("a-link"));

    let wd = WalkDir::new(dir.join(Path::new("a-link")))
        .follow_links(true)
        .follow_root_links(false);
    let r = dir.run_recursive(wd);
    assert!(r.sorted_ents().is_empty());
    assert_eq!(
        r.errs().len(),
        1,
        "broken symlink cannot be traversed - they are followed if symlinks are followed"
    );
}

#[test]
fn broken_sym_root_dir_root_is_always_followed() {
    let dir = Dir::tmp();
    dir.symlink_dir(Path::new("broken"), Path::new("a-link"));

    for follow_symlinks in &[true, false] {
        let wd =
            WalkDir::new(dir.join(Path::new("a-link"))).follow_links(*follow_symlinks);
        let r = dir.run_recursive(wd);
        assert!(r.sorted_ents().is_empty());
        assert_eq!(
            r.errs().len(),
            1,
            "broken symlink in roots cannot be traversed, they are always followed"
        );
    }
}

#[test]
fn sym_root_dir_nofollow_root_nofollow() {
    let dir = Dir::tmp();
    dir.mkdirp(Path::new("a"));
    dir.symlink_dir(Path::new("a"), Path::new("a-link"));
    dir.touch(Path::new("a/zzz"));

    let wd = WalkDir::new(dir.join(Path::new("a-link"))).follow_root_links(false);
    let r = dir.run_recursive(wd);
    r.assert_no_errors();

    let ents = r.sorted_ents();
    assert_eq!(1, ents.len());
    let link = &ents[0];
    assert_eq!(dir.join(Path::new("a-link")), link.path());
    assert_eq!(0, link.depth());
}

#[test]
fn sym_root_dir_nofollow_root_follow() {
    let dir = Dir::tmp();
    dir.mkdirp(Path::new("a"));
    dir.symlink_dir(Path::new("a"), Path::new("a-link"));
    dir.touch(Path::new("a/zzz"));

    let wd = WalkDir::new(dir.join(Path::new("a-link")));
    let r = dir.run_recursive(wd);
    r.assert_no_errors();

    let ents = r.sorted_ents();
    assert_eq!(2, ents.len());
    let link = &ents[0];

    assert_eq!(dir.join(Path::new("a-link")), link.path());

    assert!(link.path_is_symlink());

    assert_eq!(dir.join(Path::new("a")), fs::read_link(link.path()).unwrap());

    assert_eq!(0, link.depth());

    assert!(link.file_type().is_symlink());
    assert!(!link.file_type().is_file());
    assert!(!link.file_type().is_dir());

    assert!(link.metadata().unwrap().file_type().is_symlink());
    assert!(!link.metadata().unwrap().is_file());
    assert!(!link.metadata().unwrap().is_dir());

    let link_zzz = &ents[1];
    assert_eq!(dir.join(Path::new("a-link")).join(Path::new("zzz")), link_zzz.path());
    assert!(!link_zzz.path_is_symlink());
}

#[test]
fn sym_root_dir_follow() {
    let dir = Dir::tmp();
    dir.mkdirp(Path::new("a"));
    dir.symlink_dir(Path::new("a"), Path::new("a-link"));
    dir.touch(Path::new("a/zzz"));

    let wd = WalkDir::new(dir.join(Path::new("a-link"))).follow_links(true);
    let r = dir.run_recursive(wd);
    r.assert_no_errors();

    let ents = r.sorted_ents();
    assert_eq!(2, ents.len());
    let link = &ents[0];

    assert_eq!(dir.join(Path::new("a-link")), link.path());

    assert!(link.path_is_symlink());

    assert_eq!(dir.join(Path::new("a")), fs::read_link(link.path()).unwrap());

    assert_eq!(0, link.depth());

    assert!(!link.file_type().is_symlink());
    assert!(!link.file_type().is_file());
    assert!(link.file_type().is_dir());

    assert!(!link.metadata().unwrap().file_type().is_symlink());
    assert!(!link.metadata().unwrap().is_file());
    assert!(link.metadata().unwrap().is_dir());

    let link_zzz = &ents[1];
    assert_eq!(dir.join(Path::new("a-link")).join(Path::new("zzz")), link_zzz.path());
    assert!(!link_zzz.path_is_symlink());
}

#[test]
fn sym_file_nofollow() {
    let dir = Dir::tmp();
    dir.touch(Path::new("a"));
    dir.symlink_file(Path::new("a"), Path::new("a-link"));

    let wd = WalkDir::new(dir.path());
    let r = dir.run_recursive(wd);
    r.assert_no_errors();

    let ents = r.sorted_ents();
    assert_eq!(3, ents.len());
    let (src, link) = (&ents[1], &ents[2]);

    assert_eq!(dir.join(Path::new("a")), src.path());
    assert_eq!(dir.join(Path::new("a-link")), link.path());

    assert!(!src.path_is_symlink());
    assert!(link.path_is_symlink());

    assert_eq!(dir.join(Path::new("a")), fs::read_link(link.path()).unwrap());

    assert_eq!(1, src.depth());
    assert_eq!(1, link.depth());

    assert!(src.file_type().is_file());
    assert!(link.file_type().is_symlink());
    assert!(!link.file_type().is_file());
    assert!(!link.file_type().is_dir());

    assert!(src.metadata().unwrap().is_file());
    assert!(link.metadata().unwrap().file_type().is_symlink());
    assert!(!link.metadata().unwrap().is_file());
    assert!(!link.metadata().unwrap().is_dir());
}

#[test]
fn sym_file_follow() {
    let dir = Dir::tmp();
    dir.touch(Path::new("a"));
    dir.symlink_file(Path::new("a"), Path::new("a-link"));

    let wd = WalkDir::new(dir.path()).follow_links(true);
    let r = dir.run_recursive(wd);
    r.assert_no_errors();

    let ents = r.sorted_ents();
    assert_eq!(3, ents.len());
    let (src, link) = (&ents[1], &ents[2]);

    assert_eq!(dir.join(Path::new("a")), src.path());
    assert_eq!(dir.join(Path::new("a-link")), link.path());

    assert!(!src.path_is_symlink());
    assert!(link.path_is_symlink());

    assert_eq!(dir.join(Path::new("a")), fs::read_link(link.path()).unwrap());

    assert_eq!(1, src.depth());
    assert_eq!(1, link.depth());

    assert!(src.file_type().is_file());
    assert!(!link.file_type().is_symlink());
    assert!(link.file_type().is_file());
    assert!(!link.file_type().is_dir());

    assert!(src.metadata().unwrap().is_file());
    assert!(!link.metadata().unwrap().file_type().is_symlink());
    assert!(link.metadata().unwrap().is_file());
    assert!(!link.metadata().unwrap().is_dir());
}

#[test]
fn sym_dir_nofollow() {
    let dir = Dir::tmp();
    dir.mkdirp(Path::new("a"));
    dir.symlink_dir(Path::new("a"), Path::new("a-link"));
    dir.touch(Path::new("a/zzz"));

    let wd = WalkDir::new(dir.path());
    let r = dir.run_recursive(wd);
    r.assert_no_errors();

    let ents = r.sorted_ents();
    assert_eq!(4, ents.len());
    let (src, link) = (&ents[1], &ents[3]);

    assert_eq!(dir.join(Path::new("a")), src.path());
    assert_eq!(dir.join(Path::new("a-link")), link.path());

    assert!(!src.path_is_symlink());
    assert!(link.path_is_symlink());

    assert_eq!(dir.join(Path::new("a")), fs::read_link(link.path()).unwrap());

    assert_eq!(1, src.depth());
    assert_eq!(1, link.depth());

    assert!(src.file_type().is_dir());
    assert!(link.file_type().is_symlink());
    assert!(!link.file_type().is_file());
    assert!(!link.file_type().is_dir());

    assert!(src.metadata().unwrap().is_dir());
    assert!(link.metadata().unwrap().file_type().is_symlink());
    assert!(!link.metadata().unwrap().is_file());
    assert!(!link.metadata().unwrap().is_dir());
}

#[test]
fn sym_dir_follow() {
    let dir = Dir::tmp();
    dir.mkdirp(Path::new("a"));
    dir.symlink_dir(Path::new("a"), Path::new("a-link"));
    dir.touch(Path::new("a/zzz"));

    let wd = WalkDir::new(dir.path()).follow_links(true);
    let r = dir.run_recursive(wd);
    r.assert_no_errors();

    let ents = r.sorted_ents();
    assert_eq!(5, ents.len());
    let (src, link) = (&ents[1], &ents[3]);

    assert_eq!(dir.join(Path::new("a")), src.path());
    assert_eq!(dir.join(Path::new("a-link")), link.path());

    assert!(!src.path_is_symlink());
    assert!(link.path_is_symlink());

    assert_eq!(dir.join(Path::new("a")), fs::read_link(link.path()).unwrap());

    assert_eq!(1, src.depth());
    assert_eq!(1, link.depth());

    assert!(src.file_type().is_dir());
    assert!(!link.file_type().is_symlink());
    assert!(!link.file_type().is_file());
    assert!(link.file_type().is_dir());

    assert!(src.metadata().unwrap().is_dir());
    assert!(!link.metadata().unwrap().file_type().is_symlink());
    assert!(!link.metadata().unwrap().is_file());
    assert!(link.metadata().unwrap().is_dir());

    let (src_zzz, link_zzz) = (&ents[2], &ents[4]);
    assert_eq!(dir.join(Path::new("a")).join(Path::new("zzz")), src_zzz.path());
    assert_eq!(dir.join(Path::new("a-link")).join(Path::new("zzz")), link_zzz.path());
    assert!(!src_zzz.path_is_symlink());
    assert!(!link_zzz.path_is_symlink());
}

#[test]
fn sym_noloop() {
    let dir = Dir::tmp();
    dir.mkdirp(Path::new("a/b/c"));
    dir.symlink_dir(Path::new("a"), Path::new("a/b/c/a-link"));

    let wd = WalkDir::new(dir.path());
    let r = dir.run_recursive(wd);
    // There's no loop if we aren't following symlinks.
    r.assert_no_errors();

    assert_eq!(5, r.ents().len());
}

#[test]
fn sym_loop_detect() {
    let dir = Dir::tmp();
    dir.mkdirp(Path::new("a/b/c"));
    dir.symlink_dir(Path::new("a"), Path::new("a/b/c/a-link"));

    let wd = WalkDir::new(dir.path()).follow_links(true);
    let r = dir.run_recursive(wd);

    let (ents, errs) = (r.sorted_ents(), r.errs());
    assert_eq!(4, ents.len());
    assert_eq!(1, errs.len());

    let err = &errs[0];

    let expected = dir.join(Path::new("a/b/c/a-link"));
    assert_eq!(Some(&*expected), err.path());

    let expected = dir.join(Path::new("a"));
    assert_eq!(Some(&*expected), err.loop_ancestor());

    assert_eq!(4, err.depth());
    assert!(err.io_error().is_none());
}

#[test]
fn sym_self_loop_no_error() {
    let dir = Dir::tmp();
    dir.symlink_file(Path::new("a"), Path::new("a"));

    let wd = WalkDir::new(dir.path());
    let r = dir.run_recursive(wd);
    // No errors occur because even though the symlink points to nowhere, it
    // is never followed, and thus no error occurs.
    r.assert_no_errors();
    assert_eq!(2, r.ents().len());

    let ent = &r.ents()[1];
    assert_eq!(dir.join(Path::new("a")), ent.path());
    assert!(ent.path_is_symlink());

    assert!(ent.file_type().is_symlink());
    assert!(!ent.file_type().is_file());
    assert!(!ent.file_type().is_dir());

    assert!(ent.metadata().unwrap().file_type().is_symlink());
    assert!(!ent.metadata().unwrap().file_type().is_file());
    assert!(!ent.metadata().unwrap().file_type().is_dir());
}

#[test]
fn sym_file_self_loop_io_error() {
    let dir = Dir::tmp();
    dir.symlink_file(Path::new("a"), Path::new("a"));

    let wd = WalkDir::new(dir.path()).follow_links(true);
    let r = dir.run_recursive(wd);

    let (ents, errs) = (r.sorted_ents(), r.errs());
    assert_eq!(1, ents.len());
    assert_eq!(1, errs.len());

    let err = &errs[0];

    let expected = dir.join(Path::new("a"));
    assert_eq!(Some(&*expected), err.path());
    assert_eq!(1, err.depth());
    assert!(err.loop_ancestor().is_none());
    assert!(err.io_error().is_some());
}

#[test]
fn sym_dir_self_loop_io_error() {
    let dir = Dir::tmp();
    dir.symlink_dir(Path::new("a"), Path::new("a"));

    let wd = WalkDir::new(dir.path()).follow_links(true);
    let r = dir.run_recursive(wd);

    let (ents, errs) = (r.sorted_ents(), r.errs());
    assert_eq!(1, ents.len());
    assert_eq!(1, errs.len());

    let err = &errs[0];

    let expected = dir.join(Path::new("a"));
    assert_eq!(Some(&*expected), err.path());
    assert_eq!(1, err.depth());
    assert!(err.loop_ancestor().is_none());
    assert!(err.io_error().is_some());
}

#[test]
fn min_depth_1() {
    let dir = Dir::tmp();
    dir.mkdirp(Path::new("a/b"));

    let wd = WalkDir::new(dir.path()).min_depth(1);
    let r = dir.run_recursive(wd);
    r.assert_no_errors();

    let expected = vec![dir.join(Path::new("a")), dir.join(Path::new("a")).join(Path::new("b"))];
    assert_eq!(expected, r.sorted_paths());
}

#[test]
fn min_depth_2() {
    let dir = Dir::tmp();
    dir.mkdirp(Path::new("a/b"));

    let wd = WalkDir::new(dir.path()).min_depth(2);
    let r = dir.run_recursive(wd);
    r.assert_no_errors();

    let expected = vec![dir.join(Path::new("a")).join(Path::new("b"))];
    assert_eq!(expected, r.sorted_paths());
}

#[test]
fn max_depth_0() {
    let dir = Dir::tmp();
    dir.mkdirp(Path::new("a/b"));

    let wd = WalkDir::new(dir.path()).max_depth(0);
    let r = dir.run_recursive(wd);
    r.assert_no_errors();

    let expected = vec![dir.path().to_path_buf()];
    assert_eq!(expected, r.sorted_paths());
}

#[test]
fn max_depth_1() {
    let dir = Dir::tmp();
    dir.mkdirp(Path::new("a/b"));

    let wd = WalkDir::new(dir.path()).max_depth(1);
    let r = dir.run_recursive(wd);
    r.assert_no_errors();

    let expected = vec![dir.path().to_path_buf(), dir.join(Path::new("a"))];
    assert_eq!(expected, r.sorted_paths());
}

#[test]
fn max_depth_2() {
    let dir = Dir::tmp();
    dir.mkdirp(Path::new("a/b"));

    let wd = WalkDir::new(dir.path()).max_depth(2);
    let r = dir.run_recursive(wd);
    r.assert_no_errors();

    let expected =
        vec![dir.path().to_path_buf(), dir.join(Path::new("a")), dir.join(Path::new("a")).join(Path::new("b"))];
    assert_eq!(expected, r.sorted_paths());
}

// FIXME: This test seems wrong. It should return nothing!
#[test]
fn min_max_depth_diff_nada() {
    let dir = Dir::tmp();
    dir.mkdirp(Path::new("a/b/c"));

    let wd = WalkDir::new(dir.path()).min_depth(3).max_depth(2);
    let r = dir.run_recursive(wd);
    r.assert_no_errors();

    let expected = vec![dir.join(Path::new("a")).join(Path::new("b")).join(Path::new("c"))];
    assert_eq!(expected, r.sorted_paths());
}

#[test]
fn min_max_depth_diff_0() {
    let dir = Dir::tmp();
    dir.mkdirp(Path::new("a/b/c"));

    let wd = WalkDir::new(dir.path()).min_depth(2).max_depth(2);
    let r = dir.run_recursive(wd);
    r.assert_no_errors();

    let expected = vec![dir.join(Path::new("a")).join(Path::new("b"))];
    assert_eq!(expected, r.sorted_paths());
}

#[test]
fn min_max_depth_diff_1() {
    let dir = Dir::tmp();
    dir.mkdirp(Path::new("a/b/c"));

    let wd = WalkDir::new(dir.path()).min_depth(1).max_depth(2);
    let r = dir.run_recursive(wd);
    r.assert_no_errors();

    let expected = vec![dir.join(Path::new("a")), dir.join(Path::new("a")).join(Path::new("b"))];
    assert_eq!(expected, r.sorted_paths());
}

#[test]
fn contents_first() {
    let dir = Dir::tmp();
    dir.touch(Path::new("a"));

    let wd = WalkDir::new(dir.path()).contents_first(true);
    let r = dir.run_recursive(wd);
    r.assert_no_errors();

    let expected = vec![dir.join(Path::new("a")), dir.path().to_path_buf()];
    assert_eq!(expected, r.paths());
}

#[test]
fn skip_current_dir() {
    let dir = Dir::tmp();
    dir.mkdirp(Path::new("foo/bar/baz"));
    dir.mkdirp(Path::new("quux"));

    let mut paths = vec![];
    let mut it = WalkDir::new(dir.path()).into_iter();
    while let Some(result) = it.next() {
        let ent = result.unwrap();
        paths.push(ent.path().to_path_buf());
        if ent.file_name() == "bar" {
            it.skip_current_dir();
        }
    }
    paths.sort();

    let expected = vec![
        dir.path().to_path_buf(),
        dir.join(Path::new("foo")),
        dir.join(Path::new("foo")).join(Path::new("bar")),
        dir.join(Path::new("quux")),
    ];
    assert_eq!(expected, paths);
}

#[test]
fn filter_entry() {
    let dir = Dir::tmp();
    dir.mkdirp(Path::new("foo/bar/baz/abc"));
    dir.mkdirp(Path::new("quux"));

    let wd = WalkDir::new(dir.path())
        .into_iter()
        .filter_entry(|ent| ent.file_name() != "baz");
    let r = dir.run_recursive(wd);
    r.assert_no_errors();

    let expected = vec![
        dir.path().to_path_buf(),
        dir.join(Path::new("foo")),
        dir.join(Path::new("foo")).join(Path::new("bar")),
        dir.join(Path::new("quux")),
    ];
    assert_eq!(expected, r.sorted_paths());
}

#[test]
fn sort_by() {
    let dir = Dir::tmp();
    dir.mkdirp(Path::new("foo/bar/baz/abc"));
    dir.mkdirp(Path::new("quux"));

    let wd = WalkDir::new(dir.path())
        .sort_by(|a, b| a.file_name().cmp(b.file_name()).reverse());
    let r = dir.run_recursive(wd);
    r.assert_no_errors();

    let expected = vec![
        dir.path().to_path_buf(),
        dir.join(Path::new("quux")),
        dir.join(Path::new("foo")),
        dir.join(Path::new("foo")).join(Path::new("bar")),
        dir.join(Path::new("foo")).join(Path::new("bar")).join(Path::new("baz")),
        dir.join(Path::new("foo")).join(Path::new("bar")).join(Path::new("baz")).join(Path::new("abc")),
    ];
    assert_eq!(expected, r.paths());
}

#[test]
fn sort_by_key() {
    let dir = Dir::tmp();
    dir.mkdirp(Path::new("foo/bar/baz/abc"));
    dir.mkdirp(Path::new("quux"));

    let wd =
        WalkDir::new(dir.path()).sort_by_key(|a| a.file_name().to_owned());
    let r = dir.run_recursive(wd);
    r.assert_no_errors();

    let expected = vec![
        dir.path().to_path_buf(),
        dir.join(Path::new("foo")),
        dir.join(Path::new("foo")).join(Path::new("bar")),
        dir.join(Path::new("foo")).join(Path::new("bar")).join(Path::new("baz")),
        dir.join(Path::new("foo")).join(Path::new("bar")).join(Path::new("baz")).join(Path::new("abc")),
        dir.join(Path::new("quux")),
    ];
    assert_eq!(expected, r.paths());
}

#[test]
fn sort_by_file_name() {
    let dir = Dir::tmp();
    dir.mkdirp(Path::new("foo/bar/baz/abc"));
    dir.mkdirp(Path::new("quux"));

    let wd = WalkDir::new(dir.path()).sort_by_file_name();
    let r = dir.run_recursive(wd);
    r.assert_no_errors();

    let expected = vec![
        dir.path().to_path_buf(),
        dir.join(Path::new("foo")),
        dir.join(Path::new("foo")).join(Path::new("bar")),
        dir.join(Path::new("foo")).join(Path::new("bar")).join(Path::new("baz")),
        dir.join(Path::new("foo")).join(Path::new("bar")).join(Path::new("baz")).join(Path::new("abc")),
        dir.join(Path::new("quux")),
    ];
    assert_eq!(expected, r.paths());
}

#[test]
fn sort_max_open() {
    let dir = Dir::tmp();
    dir.mkdirp(Path::new("foo/bar/baz/abc"));
    dir.mkdirp(Path::new("quux"));

    let wd = WalkDir::new(dir.path())
        .max_open(1)
        .sort_by(|a, b| a.file_name().cmp(b.file_name()).reverse());
    let r = dir.run_recursive(wd);
    r.assert_no_errors();

    let expected = vec![
        dir.path().to_path_buf(),
        dir.join(Path::new("quux")),
        dir.join(Path::new("foo")),
        dir.join(Path::new("foo")).join(Path::new("bar")),
        dir.join(Path::new("foo")).join(Path::new("bar")).join(Path::new("baz")),
        dir.join(Path::new("foo")).join(Path::new("bar")).join(Path::new("baz")).join(Path::new("abc")),
    ];
    assert_eq!(expected, r.paths());
}

#[cfg(target_os = "linux")]
#[test]
fn same_file_system() {
    use std::path::Path;

    // This test is a little weird since it's not clear whether it's a good
    // idea to setup a distinct mounted volume in these tests. Instead, we
    // probe for an existing one.
    if !Path::new("/sys").is_dir() {
        return;
    }

    let dir = Dir::tmp();
    dir.touch(Path::new("a"));
    dir.symlink_dir(Path::new("/sys"), Path::new("sys-link"));

    // First, do a sanity check that things work without following symlinks.
    let wd = WalkDir::new(dir.path());
    let r = dir.run_recursive(wd);
    r.assert_no_errors();

    let expected =
        vec![dir.path().to_path_buf(), dir.join(Path::new("a")), dir.join(Path::new("sys-link"))];
    assert_eq!(expected, r.sorted_paths());

    // ... now follow symlinks and ensure we don't descend into /sys.
    let wd =
        WalkDir::new(dir.path()).same_file_system(true).follow_links(true);
    let r = dir.run_recursive(wd);
    r.assert_no_errors();

    let expected =
        vec![dir.path().to_path_buf(), dir.join(Path::new("a")), dir.join(Path::new("sys-link"))];
    assert_eq!(expected, r.sorted_paths());
}

// Tests that skip_current_dir doesn't destroy internal invariants.
//
// See: https://github.com/BurntSushi/walkdir/issues/118
#[test]
fn regression_skip_current_dir() {
    let dir = Dir::tmp();
    dir.mkdirp(Path::new("foo/a/b"));
    dir.mkdirp(Path::new("foo/1/2"));

    let mut wd = WalkDir::new(dir.path()).max_open(1).into_iter();
    wd.next();
    wd.next();
    wd.next();
    wd.next();

    wd.skip_current_dir();
    wd.skip_current_dir();
    wd.next();
}
