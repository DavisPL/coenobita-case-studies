use crate::error::{Error, Result};
use crate::manifest::Name;
use crate::Test;
use std::collections::BTreeMap as Map;
use std::ffi::OsStr;
use std::iter::Peekable;
use std::path::{Components, Path, PathBuf};

#[derive(Debug)]
pub(crate) struct ExpandedTest {
    pub name: Name,
    pub test: Test,
    pub error: Option<Error>,
    is_from_glob: bool,
}

pub(crate) fn expand_globs(tests: &[Test], collect: Collect, join: Join, generate: Generate) -> Vec<ExpandedTest> {
    let mut set = ExpandedTestSet::new();

    for test in tests {
        match test.path.to_str() {
            Some(utf8) if utf8.contains('*') => match glob(utf8, collect, join, generate) {
                Ok(paths) => {
                    let expected = test.expected;
                    for path in paths {
                        set.insert(Test { path, expected }, None, true);
                    }
                }
                Err(error) => set.insert(test.clone(), Some(error), false),
            },
            _ => set.insert(test.clone(), None, false),
        }
    }

    set.vec
}

struct ExpandedTestSet {
    vec: Vec<ExpandedTest>,
    path_to_index: Map<PathBuf, usize>,
}

impl ExpandedTestSet {
    fn new() -> Self {
        ExpandedTestSet {
            vec: Vec::new(),
            path_to_index: Map::new(),
        }
    }

    fn insert(&mut self, test: Test, error: Option<Error>, is_from_glob: bool) {
        if let Some(&i) = self.path_to_index.get(&test.path) {
            let prev = &mut self.vec[i];
            if prev.is_from_glob {
                prev.test.expected = test.expected;
                return;
            }
        }

        let index = self.vec.len();
        let name = Name(format!("trybuild{:03}", index));
        self.path_to_index.insert(test.path.clone(), index);
        self.vec.push(ExpandedTest {
            name,
            test,
            error,
            is_from_glob,
        });
    }
}

type Collect = fn(Peekable<Components>) -> PathBuf;
type Generate = fn(&OsStr) -> &Path;
type Join = fn(&OsStr, &OsStr) -> PathBuf;

fn glob(pattern: &str, collect: Collect, join: Join, generate: Generate) -> Result<Vec<PathBuf>> {
    let mut paths = glob::glob(pattern, collect, generate, join)?
        .map(|entry| entry.map_err(Error::from))
        .collect::<Result<Vec<PathBuf>>>()?;
    paths.sort();
    Ok(paths)
}
