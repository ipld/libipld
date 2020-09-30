//! Path
use crate::cid::{Cid, Size};

/// Represents a path in an ipld dag.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Path(Vec<String>);

impl Path {
    /// Iterate over path segments.
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(|s| &**s)
    }

    /// Join segment.
    pub fn join<T: AsRef<str>>(&mut self, segment: T) {
        for seg in segment.as_ref().split('/').filter(|s| !s.is_empty()) {
            self.0.push(seg.to_owned())
        }
    }
}

impl From<Vec<String>> for Path {
    fn from(segments: Vec<String>) -> Self {
        Path(segments)
    }
}

impl From<Vec<&str>> for Path {
    fn from(segments: Vec<&str>) -> Self {
        Path(segments.into_iter().map(String::from).collect())
    }
}

impl From<&str> for Path {
    fn from(s: &str) -> Self {
        let mut path = Path::default();
        path.join(s);
        path
    }
}

impl From<String> for Path {
    fn from(s: String) -> Self {
        Path::from(s.as_str())
    }
}

impl ToString for Path {
    fn to_string(&self) -> String {
        let mut path = "".to_string();
        for seg in &self.0 {
            path.push_str(seg.as_str());
            path.push_str("/");
        }
        path.pop();
        path
    }
}

/// Path in a dag.
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct DagPath<'a, S: Size>(&'a Cid<S>, Path);

impl<'a, S: Size> DagPath<'a, S> {
    /// Create a new dag path.
    pub fn new<T: Into<Path>>(cid: &'a Cid<S>, path: T) -> Self {
        Self(cid, path.into())
    }

    /// Returns the root of the path.
    pub fn root(&self) -> &Cid<S> {
        self.0
    }

    /// Returns the ipld path.
    pub fn path(&self) -> &Path {
        &self.1
    }
}

impl<'a, S: Size> From<&'a Cid<S>> for DagPath<'a, S> {
    fn from(cid: &'a Cid<S>) -> Self {
        Self(cid, Default::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing_one_segment() {
        assert_eq!(Path::from("0"), Path::from(vec!["0"]));
    }

    #[test]
    fn test_parsing_three_segments() {
        assert_eq!(Path::from("0/foo/2"), Path::from(vec!["0", "foo", "2"]));
    }

    #[test]
    fn test_eliding_empty_segments() {
        assert_eq!(Path::from("0//2"), Path::from(vec!["0", "2"]));
    }

    #[test]
    fn test_eliding_leading_slashes() {
        assert_eq!(Path::from("/0/2"), Path::from(vec!["0", "2"]));
    }

    #[test]
    fn test_eliding_trailing_slashes() {
        assert_eq!(Path::from("0/2/"), Path::from(vec!["0", "2"]));
    }

    #[test]
    fn test_to_string() {
        assert_eq!(Path::from(vec!["0", "foo", "2"]).to_string(), "0/foo/2");
    }
}
