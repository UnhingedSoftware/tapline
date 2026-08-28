//! A small glob matcher, for selecting entries out of an archive.

/// Whether `path` matches `pattern`.
#[must_use]
pub fn matches(pattern: &str, path: &str) -> bool {
    let pattern: Vec<char> = pattern.to_ascii_lowercase().chars().collect();
    let path: Vec<char> = path.to_ascii_lowercase().chars().collect();
    matches_from(&pattern, 0, &path, 0)
}

fn matches_from(pattern: &[char], mut p: usize, path: &[char], mut s: usize) -> bool {
    while p < pattern.len() {
        match pattern.get(p) {
            Some('*') => {
                let double = pattern.get(p + 1) == Some(&'*');
                let next = if double { p + 2 } else { p + 1 };

                if double && next >= pattern.len() {
                    return true;
                }
                // Try every split point; `*` may not cross a separator, `**` may.
                let mut at = s;
                loop {
                    if matches_from(pattern, next, path, at) {
                        return true;
                    }
                    let Some(ch) = path.get(at) else {
                        return false;
                    };
                    if !double && *ch == '/' {
                        return false;
                    }
                    at += 1;
                }
            }
            Some('?') => {
                match path.get(s) {
                    Some('/') | None => return false,
                    Some(_) => {}
                }
                p += 1;
                s += 1;
            }
            Some(expected) => {
                if path.get(s) != Some(expected) {
                    return false;
                }
                p += 1;
                s += 1;
            }
            None => break,
        }
    }
    s == path.len()
}

/// A set of patterns. Empty matches everything.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Patterns {
    patterns: Vec<String>,
}

impl Patterns {
    /// A set that matches everything.
    #[must_use]
    pub const fn all() -> Self {
        Self {
            patterns: Vec::new(),
        }
    }

    /// Adds a pattern.
    #[must_use]
    pub fn with(mut self, pattern: impl Into<String>) -> Self {
        self.patterns.push(pattern.into());
        self
    }

    /// Whether nothing was specified, and so everything matches.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }

    /// The patterns, for encoding.
    #[must_use]
    pub fn as_slice(&self) -> &[String] {
        &self.patterns
    }

    /// Whether `path` is selected; an empty set selects everything.
    #[must_use]
    pub fn selects(&self, path: &str) -> bool {
        self.patterns.is_empty() || self.patterns.iter().any(|pattern| matches(pattern, path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_literal_matches_itself_and_nothing_else() {
        assert!(matches("lua/init.lua", "lua/init.lua"));
        assert!(!matches("lua/init.lua", "lua/init.luax"));
        assert!(!matches("lua/init.lua", "lua/other.lua"));
    }

    #[test]
    fn a_star_stops_at_a_separator() {
        assert!(matches("*.lua", "init.lua"));
        assert!(!matches("*.lua", "lua/init.lua"));
        assert!(matches("lua/*.lua", "lua/init.lua"));
        assert!(!matches("lua/*.lua", "lua/deep/init.lua"));
    }

    #[test]
    fn a_double_star_crosses_separators() {
        assert!(matches("lua/**", "lua/init.lua"));
        assert!(matches("lua/**", "lua/pac3/extra/client/x.lua"));
        assert!(!matches("lua/**", "materials/x.vmt"));
        assert!(matches("**/*.vmt", "materials/models/x.vmt"));
        assert!(matches("**", "anything/at/all"));
    }

    #[test]
    fn a_directory_pattern_does_not_match_a_file_of_that_name() {
        assert!(!matches("lua/**", "lua"));
        assert!(matches("lua/**", "lua/a.lua"));
    }

    #[test]
    fn a_question_mark_is_exactly_one_character() {
        assert!(matches("a?c", "abc"));
        assert!(!matches("a?c", "ac"));
        assert!(!matches("a?c", "abbc"));
        assert!(!matches("a?c", "a/c"), "? must not cross a separator");
    }

    #[test]
    fn matching_ignores_case() {
        assert!(matches("lua/**", "LUA/Init.lua"));
        assert!(matches("*.VMT", "x.vmt"));
    }

    #[test]
    fn an_empty_pattern_set_selects_everything() {
        let all = Patterns::all();
        assert!(all.is_empty());
        assert!(all.selects("anything"));
        assert!(all.selects(""));
    }

    #[test]
    fn any_pattern_matching_is_enough() {
        let some = Patterns::all().with("lua/**").with("*.txt");
        assert!(some.selects("lua/a.lua"));
        assert!(some.selects("readme.txt"));
        assert!(!some.selects("materials/x.vmt"));
    }

    #[test]
    fn a_pattern_that_matches_nothing_is_not_an_error() {
        let none = Patterns::all().with("nothing/**");
        assert!(!none.selects("lua/a.lua"));
    }

    #[test]
    fn pathological_patterns_terminate() {
        let start = std::time::Instant::now();
        let _ = matches("**/**/**/**/*x", "a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p");
        assert!(
            start.elapsed().as_millis() < 500,
            "matching took {:?}",
            start.elapsed()
        );
    }
}
