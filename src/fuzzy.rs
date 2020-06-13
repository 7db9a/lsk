use fuzzy_matcher;

pub mod demo {
    use std::path::PathBuf;
    use fuzzy_matcher::FuzzyMatcher;
    use fuzzy_matcher::skim::SkimMatcherV2;

    #[derive(Debug, Clone, PartialEq)]
    pub enum Score {
         Files((PathBuf, Option<(i64, Vec<usize>)>)),
         Dirs((PathBuf, Option<(i64, Vec<usize>)>)),
    }

    impl Score {
        pub fn score(&self) -> (PathBuf, Option<(i64, Vec<usize>)>) {
            match self.clone() {
                Score::Files(score) => score,
                Score::Dirs(score) => score,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Scores {
        pub files: Vec<Score>,
        pub dirs: Vec<Score>
    }

    pub fn score(compare_to: &str, guess: &str) -> Option<(i64, Vec<usize>)> {
        let matcher = SkimMatcherV2::default();
        matcher.fuzzy_indices(compare_to, guess)
    }

    pub fn order(a: &Score, b: &Score) -> std::cmp::Ordering {
        let a_score = a.score().0;
        let b_score = b.score().0;

        if a_score == b_score {
            std::cmp::Ordering::Equal
        } else if a_score > b_score {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use super::*;
    #[test]
    #[ignore]//docker
    fn score() {
        let res = super::demo::score("abc", "abx");
        assert_eq!(res, None);
        let (score, indices) = super::demo::score("axbycz", "xyz").unwrap();
        assert_eq!(indices, [1, 3, 5]);
        assert_eq!(score, 39);
        let (score, indices) = super::demo::score("axbycz", "abc").unwrap();
        assert_eq!(indices, [0, 2, 4]);
        assert_eq!(score, 55);
        let (score, indices) = super::demo::score("unignore_play_test", "uigp").unwrap();
        assert_eq!(indices, [0, 2, 3, 9]);
        assert_eq!(score, 78);
    }

    #[test]
    #[ignore]//docker
    fn sort_score() {
        let guess = "xyz";
        let file_a = "xayb";
        let file_b = "xyazabc";
        let file_c = "xyza";
        let file_d = "afd";
        let dir_a = "dirxyzabc";
        let dir_b = "dirxzabc";

        let res_a = super::demo::score(file_a, guess);
        let res_b = super::demo::score(file_b, guess);
        let res_c = super::demo::score(file_c, guess);
        let res_d = super::demo::score(file_d, guess);

        let mut scores = super::demo::Scores {
            files: vec![
                     super::demo::Score::Files((PathBuf::from(file_a), res_a.clone())),
                     super::demo::Score::Files((PathBuf::from(file_b), res_b.clone())),
                     super::demo::Score::Files((PathBuf::from(file_c), res_c.clone())),
                     super::demo::Score::Files((PathBuf::from(file_d), res_d.clone())),
                 ],

            dirs: vec![
                     super::demo::Score::Files((PathBuf::from(file_a), res_a.clone())),
                     super::demo::Score::Files((PathBuf::from(file_b), res_b.clone())),
                 ]
        };

        let pre_sort = scores.clone();
        scores.files.sort_by(|a, b| demo::order(a, b));
        let post_sort = scores.clone();
        assert_ne!(
            pre_sort,
            post_sort
        );
        //scores.dirs.sort_by(|a, b| demo::order(a, b));

        assert_ne!(
            pre_sort.files,
            post_sort.files
        );

        assert_eq!(
             scores.files.iter().count(),
             4
        );

        assert_eq!(
            scores.files.iter().nth(0).unwrap().score().0,
            PathBuf::from("xyza")
        );

        assert_eq!(
            scores.files.iter().nth(1).unwrap().score().0,
            PathBuf::from("xyazabc")
        );

        assert_eq!(
            scores.files.iter().nth(2).unwrap().score().0,
            PathBuf::from("xayb")
        )
    }
}
