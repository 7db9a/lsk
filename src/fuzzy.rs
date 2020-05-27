use fuzzy_matcher;

pub mod demo {
    use fuzzy_matcher::FuzzyMatcher;
    use fuzzy_matcher::skim::SkimMatcherV2;

    pub fn score() {
        let matcher = SkimMatcherV2::default();
        assert_eq!(None, matcher.fuzzy_match("abc", "abx"));
        assert!(matcher.fuzzy_match("axbycz", "abc").is_some());
        assert!(matcher.fuzzy_match("axbycz", "xyz").is_some());

        let (score, indices) = matcher.fuzzy_indices("axbycz", "abc").unwrap();
        assert_eq!(indices, [0, 2, 4]);
        assert_eq!(score, 70);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore]//docker
    fn score() {
        super::demo::score()
    }

    //#[ignore]//host
}
