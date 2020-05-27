use fuzzy_matcher;

pub mod demo {
    use fuzzy_matcher::FuzzyMatcher;
    use fuzzy_matcher::skim::SkimMatcherV2;

    pub fn score(compare_to: &'static str, guess: &'static str) -> Option<(i64, Vec<usize>)> {
        let matcher = SkimMatcherV2::default();
        matcher.fuzzy_indices(compare_to, guess)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore]//docker
    fn score() {
        let res = super::demo::score("abc", "abx");
        assert_eq!(res, None)
        //super::demo::score("abcycz", "abc")
        //super::demo::score("abcycz", "xyz")
        //assert_eq!(indices, [0, 2, 4]);
        //assert_eq!(score, 70);
    }

    //#[ignore]//host
}
