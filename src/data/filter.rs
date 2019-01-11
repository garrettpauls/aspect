use std::default::Default;

use super::File;

#[derive(Debug, Clone)]
pub struct Filter {
    /// The file name should contain this. This value should always be lowercase.
    file_name: Option<String>,
}

// Builder methods
impl Filter {
    pub fn with_name(mut self, name: &str) -> Self {
        self.file_name = if name.is_empty() { None } else { Some(name.to_lowercase()) };
        self
    }
}

impl Default for Filter {
    fn default() -> Self {
        Filter {
            file_name: None
        }
    }
}


// Utility
impl Filter {
    pub fn is_subset_of(&self, other: &Filter) -> bool {
        let is_file_name_subset = match (&self.file_name, &other.file_name) {
            (None, None) => true,
            (Some(_), None) => true,
            (None, Some(_)) => false,
            (Some(new), Some(current)) => new.starts_with(current)
        };

        is_file_name_subset
    }

    pub fn matches(&self, file: &File) -> bool {
        let name_matches = match (&self.file_name, file.path.file_name()) {
            (Some(_), None) => false,
            (None, _) => true,
            (Some(filter), Some(name)) => name.to_str().map(|s| s.to_lowercase().contains(&*filter)).unwrap_or(false)
        };

        name_matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::File;
    use std::path::PathBuf;

    #[test]
    pub fn is_subset_of_by_name() {
        let current = Filter::default().with_name("abc");
        test_is_subset_of(&Filter::default().with_name(""), &current, false);
        test_is_subset_of(&Filter::default().with_name("a"), &current, false);
        test_is_subset_of(&Filter::default().with_name("abc"), &current, true);
        test_is_subset_of(&Filter::default().with_name("abcd"), &current, true);
        test_is_subset_of(&Filter::default().with_name(""), &Filter::default(), true);
        test_is_subset_of(&Filter::default().with_name("a"), &Filter::default(), true);
    }

    fn test_is_subset_of(filter: &Filter, current: &Filter, expected: bool) {
        assert_eq!(filter.is_subset_of(current), expected, "{:?} is subset of {:?}", filter, current);
    }

    #[test]
    pub fn matches_by_name() {
        let file = File {
            path: PathBuf::from(r"C:\path\to\file.png"),
            rating: None,
        };

        test_matches(&Filter::default(), &file, true);
        test_matches(&Filter::default().with_name("f"), &file, true);
        test_matches(&Filter::default().with_name("file"), &file, true);
        test_matches(&Filter::default().with_name("file."), &file, true);
        test_matches(&Filter::default().with_name("file.p"), &file, true);
        test_matches(&Filter::default().with_name("file.png"), &file, true);
        test_matches(&Filter::default().with_name("FILE.PNG"), &file, true);
        test_matches(&Filter::default().with_name("FILE"), &file, true);
        test_matches(&Filter::default().with_name("FI"), &file, true);
        test_matches(&Filter::default().with_name("file.pnga"), &file, false);
        test_matches(&Filter::default().with_name("file*png"), &file, false);
        test_matches(&Filter::default().with_name("*file.png"), &file, false);
        test_matches(&Filter::default().with_name("other.png"), &file, false);
    }

    fn test_matches(filter: &Filter, file: &File, expected: bool) {
        assert_eq!(filter.matches(file), expected, "{:?} matches {:?}", filter, file);
    }
}