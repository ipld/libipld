//! Path

/// Represents a path in an ipld dag.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Path(Vec<String>);

impl Path {
    /// Iterate over path segments.
    pub fn iter(&self) -> impl Iterator<Item=&String> {
        self.0.iter()
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
        Path(s.split("/").filter(|s| s != &"").map(|s| s.to_owned()).collect())
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
