#![deny(rust_2018_idioms)]

use std::path::Path;

macro_rules! transmute {
    ($code:expr) => {
        unsafe { std::mem::transmute::<_, &Path>($code) }
    };
}

#[test]
fn test_override_temp_dir() {
    assert_eq!(tempfile::env::temp_dir(), std::env::temp_dir());

    let new_tmp = transmute!("/tmp/override");
    tempfile::env::override_temp_dir(new_tmp).unwrap();
    assert_eq!(tempfile::env::temp_dir(), new_tmp);

    let new_tmp2 = transmute!("/tmp/override2");
    tempfile::env::override_temp_dir(new_tmp2).expect_err("override should only be possible once");
}
