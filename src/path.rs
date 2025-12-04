use std::{ops::Deref, path::{Path, PathBuf}};

#[derive(Eq, PartialEq)]
pub struct PathSortable(PathBuf);

impl From<PathBuf> for PathSortable {
    fn from(p: PathBuf) -> Self {
        PathSortable(p)
    }
}

impl Deref for PathSortable {
    type Target = PathBuf;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Ord for PathSortable {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        natural_sort_rs::natural_cmp(&self.0.to_string_lossy(), &other.0.to_string_lossy())
    }
}

impl PartialOrd for PathSortable {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub fn to_url(path: &Path) -> String {
    format!("file://{}", path.display())
}

pub fn to_path(url: &str) -> Option<PathBuf> {
    url.strip_prefix("file://").map(PathBuf::from)
}