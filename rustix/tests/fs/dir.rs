#[test]
fn test_dir_read_from() {
    let t = rustix::fs::openat(
        rustix::fs::CWD,
        rustix::cstr!("."),
        rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::empty(),
    )
    .unwrap();

    let mut dir = rustix::fs::Dir::read_from(&t).unwrap();

    let _file = rustix::fs::openat(
        &t,
        rustix::cstr!("Cargo.toml"),
        rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::empty(),
    )
    .unwrap();

    // Read the directory entries. We use `while let Some(entry)` so that we
    // don't consume the `Dir` so that we can run more tests on it.
    let mut saw_dot = false;
    let mut saw_dotdot = false;
    let mut saw_cargo_toml = false;
    while let Some(entry) = dir.read() {
        let entry = entry.unwrap();
        if entry.file_name() == rustix::cstr!(".") {
            saw_dot = true;
        } else if entry.file_name() == rustix::cstr!("..") {
            saw_dotdot = true;
        } else if entry.file_name() == rustix::cstr!("Cargo.toml") {
            saw_cargo_toml = true;
        }
    }
    assert!(saw_dot);
    assert!(saw_dotdot);
    assert!(saw_cargo_toml);

    // Rewind the directory so we can iterate over the entries again.
    dir.rewind();

    // For what comes next, we don't need `mut` anymore.
    let dir = dir;

    // Read the directory entries, again. This time we use `for entry in dir`.
    let mut saw_dot = false;
    let mut saw_dotdot = false;
    let mut saw_cargo_toml = false;
    for entry in dir {
        let entry = entry.unwrap();
        if entry.file_name() == rustix::cstr!(".") {
            saw_dot = true;
        } else if entry.file_name() == rustix::cstr!("..") {
            saw_dotdot = true;
        } else if entry.file_name() == rustix::cstr!("Cargo.toml") {
            saw_cargo_toml = true;
        }
    }
    assert!(saw_dot);
    assert!(saw_dotdot);
    assert!(saw_cargo_toml);
}

#[test]
fn test_dir_new() {
    let t = rustix::fs::openat(
        rustix::fs::CWD,
        rustix::cstr!("."),
        rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::empty(),
    )
    .unwrap();

    let _file = rustix::fs::openat(
        &t,
        rustix::cstr!("Cargo.toml"),
        rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::empty(),
    )
    .unwrap();

    let mut dir = rustix::fs::Dir::new(t).unwrap();

    // Read the directory entries. We use `while let Some(entry)` so that we
    // don't consume the `Dir` so that we can run more tests on it.
    let mut saw_dot = false;
    let mut saw_dotdot = false;
    let mut saw_cargo_toml = false;
    while let Some(entry) = dir.read() {
        let entry = entry.unwrap();
        if entry.file_name() == rustix::cstr!(".") {
            saw_dot = true;
        } else if entry.file_name() == rustix::cstr!("..") {
            saw_dotdot = true;
        } else if entry.file_name() == rustix::cstr!("Cargo.toml") {
            saw_cargo_toml = true;
        }
    }
    assert!(saw_dot);
    assert!(saw_dotdot);
    assert!(saw_cargo_toml);

    // Rewind the directory so we can iterate over the entries again.
    dir.rewind();

    // For what comes next, we don't need `mut` anymore.
    let dir = dir;

    // Read the directory entries, again. This time we use `for entry in dir`.
    let mut saw_dot = false;
    let mut saw_dotdot = false;
    let mut saw_cargo_toml = false;
    for entry in dir {
        let entry = entry.unwrap();
        if entry.file_name() == rustix::cstr!(".") {
            saw_dot = true;
        } else if entry.file_name() == rustix::cstr!("..") {
            saw_dotdot = true;
        } else if entry.file_name() == rustix::cstr!("Cargo.toml") {
            saw_cargo_toml = true;
        }
    }
    assert!(saw_dot);
    assert!(saw_dotdot);
    assert!(saw_cargo_toml);
}

// Test that `Dir` silently stops iterating if the directory has been removed.
//
// Except on FreeBSD and macOS, where apparently `readdir` just keeps reading.
#[cfg_attr(any(apple, freebsdlike), ignore)]
#[test]
fn dir_iterator_handles_dir_removal() {
    // create a dir, keep the FD, then delete the dir
    let tmp = tempfile::tempdir(|p1, p2| std::path::Path::new(p1).join(p2)).unwrap();
    let fd = rustix::fs::openat(
        rustix::fs::CWD,
        tmp.path(),
        rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::empty(),
    )
    .unwrap();

    // Drop the `TempDir`, which deletes the directory.
    drop(tmp);

    let mut dir = rustix::fs::Dir::read_from(&fd).unwrap();
    assert!(dir.next().is_none());
}

// Like `dir_iterator_handles_dir_removal`, but close the directory after
// `Dir::read_from`.
#[cfg_attr(any(apple, freebsdlike), ignore)]
#[test]
fn dir_iterator_handles_dir_removal_after_open() {
    // create a dir, keep the FD, then delete the dir
    let tmp = tempfile::tempdir(|p1, p2| std::path::Path::new(p1).join(p2)).unwrap();
    let fd = rustix::fs::openat(
        rustix::fs::CWD,
        tmp.path(),
        rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::empty(),
    )
    .unwrap();

    let mut dir = rustix::fs::Dir::read_from(&fd).unwrap();

    // Drop the `TempDir`, which deletes the directory.
    drop(tmp);

    assert!(dir.next().is_none());
}
