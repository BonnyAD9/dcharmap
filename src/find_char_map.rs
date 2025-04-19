use std::collections::{HashMap, HashSet};

use itertools::Itertools;

pub struct WordTree<'a> {
    pub word: &'a str,
    pub next: Vec<WordTree<'a>>,
}

pub struct FcmData {
    orig_words: Vec<String>,
    word_map: Vec<usize>,
    dict: HashMap<usize, Vec<String>>,
    words: Vec<Word>,
}

struct Word {
    rel: Vec<usize>,
    clen: usize,
}

impl FcmData {
    pub fn find_char_map(&self) -> Vec<WordTree> {
        self.find_char_map_inner(0, HashMap::new())
            .unwrap_or_default()
    }

    pub fn find_char_map_inner(
        &self,
        depth: usize,
        mut umap: HashMap<char, usize>,
    ) -> Option<Vec<WordTree>> {
        let words = &self.words[depth..];

        if words.is_empty() {
            return Some(vec![]);
        }

        let mut urep = vec![];

        let word = &words[0];

        let opts = self.dict.get(&word.clen)?;

        let len = umap.len();

        let mut res = vec![];

        for opt in opts {
            relative_representation(opt, &mut urep, &mut umap);
            if urep == word.rel {
                let branches =
                    self.find_char_map_inner(depth + 1, umap.clone());
                if let Some(next) = branches {
                    let tree = WordTree { word: opt, next };
                    res.push(tree)
                }
            }
            umap.retain(|_, e| *e < len);
        }

        (!res.is_empty()).then_some(res)
    }

    pub fn word_map(&self) -> &[usize] {
        &self.word_map
    }

    pub fn new<E>(
        words: Vec<String>,
        di: impl Iterator<Item = Result<String, E>>,
    ) -> Result<Self, E> {
        let mut res = Self {
            orig_words: vec![],
            word_map: vec![],
            dict: HashMap::new(),
            words: vec![],
        };

        res.set_words(words);
        res.load_dict(di)?;

        Ok(res)
    }

    fn set_words(&mut self, words: Vec<String>) {
        self.orig_words = words;
        let mut procw = self
            .orig_words
            .iter()
            .map(|w| w.as_str())
            .unique()
            .collect_vec();
        procw.sort_unstable();

        self.word_map = self
            .orig_words
            .iter()
            .map(|w| {
                for (i, pw) in procw.iter().enumerate() {
                    if pw == w {
                        return i;
                    }
                }
                unreachable!()
            })
            .collect();

        let mut rel_map = HashMap::new();
        for word in procw {
            let mut rel = vec![];
            relative_representation(word, &mut rel, &mut rel_map);
            self.words.push(Word {
                rel,
                clen: word.len(),
            });
        }
    }

    fn load_dict<E>(
        &mut self,
        i: impl Iterator<Item = Result<String, E>>,
    ) -> Result<(), E> {
        let lengths: HashSet<_> =
            self.orig_words.iter().map(|a| a.len()).unique().collect();
        for s in i {
            let s = s?;
            let s = s.trim();
            if !lengths.contains(&s.len()) {
                continue;
            }

            self.dict.entry(s.len()).or_default().push(s.to_string());
        }

        Ok(())
    }
}

impl<'a> WordTree<'a> {
    pub fn walk(&self, mut f: impl FnMut(&[&'a str])) {
        self.walk_inner(&mut vec![], &mut f);
    }

    fn walk_inner(
        &self,
        sentance: &mut Vec<&'a str>,
        f: &mut impl FnMut(&[&'a str]),
    ) {
        sentance.push(self.word);
        if self.next.is_empty() {
            f(sentance);
            sentance.pop();
            return;
        }

        for w in &self.next {
            w.walk_inner(sentance, f);
        }

        sentance.pop();
    }
}

fn relative_representation(
    s: &str,
    res: &mut Vec<usize>,
    map: &mut HashMap<char, usize>,
) {
    res.clear();
    res.extend(s.chars().map(|c| {
        let len = map.len();
        *map.entry(c).or_insert(len)
    }));
}
