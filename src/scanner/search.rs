use aho_corasick::AhoCorasickBuilder;
use anyhow::{Context, Result};
use std::collections::HashSet;

pub struct AhoMatcher {
    ac: aho_corasick::AhoCorasick,
    originals: Vec<String>,
    max_pattern_len: usize,
}

impl AhoMatcher {
    pub fn new(patterns: &[String]) -> Result<Self> {
        let mut uniques = Vec::new();
        let mut seen = HashSet::new();
        for p in patterns {
            let trimmed = p.trim();
            if trimmed.is_empty() {
                continue;
            }
            let lower = trimmed.to_ascii_lowercase();
            if seen.insert(lower.clone()) {
                uniques.push((trimmed.to_string(), lower));
            }
        }

        let originals = uniques.iter().map(|(o, _)| o.clone()).collect::<Vec<_>>();
        let lowercased = uniques.iter().map(|(_, l)| l.clone()).collect::<Vec<_>>();
        let max_pattern_len = lowercased.iter().map(|p| p.len()).max().unwrap_or(0);

        let ac = AhoCorasickBuilder::new()
            .ascii_case_insensitive(true)
            .build(&lowercased)
            .context("Failed to build Aho-Corasick matcher")?;

        Ok(Self {
            ac,
            originals,
            max_pattern_len,
        })
    }

    pub fn find_indices(&self, haystack_lower: &str) -> Vec<usize> {
        self.ac
            .find_iter(haystack_lower)
            .map(|m| m.pattern().as_usize())
            .collect()
    }

    pub fn pattern(&self, idx: usize) -> &str {
        self.originals
            .get(idx)
            .map(|s| s.as_str())
            .unwrap_or_default()
    }

    pub fn patterns_len(&self) -> usize {
        self.originals.len()
    }

    pub fn max_pattern_len(&self) -> usize {
        self.max_pattern_len.max(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_case_insensitive() {
        let matcher = AhoMatcher::new(&["Token".to_string(), "user".to_string()]).unwrap();
        let hay = "bearer token=123".to_ascii_lowercase();
        let indices = matcher.find_indices(&hay);
        assert_eq!(indices.len(), 1);
        assert_eq!(matcher.pattern(indices[0]), "Token");
    }
}
