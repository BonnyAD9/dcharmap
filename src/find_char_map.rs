use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use itertools::Itertools;

pub struct WordTree<'a> {
    pub word: &'a str,
    pub next: Vec<WordTree<'a>>,
}

pub struct FcmData {
    orig_words: Vec<String>,
    word_map: Vec<usize>,
    dict: HashMap<Vec<usize>, Vec<String>>,
    words: Vec<Word>,
}

struct Word {
    rel: Vec<usize>,
    urel: Vec<usize>,
}

impl FcmData {
    pub fn find_char_map(&self) -> Vec<WordTree> {
        self.find_char_map_inner(0, HashMap::new())
            .unwrap_or_default()
    }

    pub fn find_char_map_inner(
        &self,
        depth: usize,
        mut rmap: HashMap<char, usize>,
    ) -> Option<Vec<WordTree>> {
        let words = &self.words[depth..];

        if words.is_empty() {
            return Some(vec![]);
        }

        let mut rel = vec![];

        let word = &words[0];

        let opts = self.dict.get(&word.urel)?;

        let len = rmap.len();

        let mut res = vec![];

        for opt in opts {
            relative_representation(opt.chars(), &mut rel, &mut rmap);
            if rel == word.rel {
                let branches =
                    self.find_char_map_inner(depth + 1, rmap.clone());
                if let Some(next) = branches {
                    let tree = WordTree { word: opt, next };
                    res.push(tree)
                }
            }
            rmap.retain(|_, e| *e < len);
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
        let mut buf = HashMap::new();
        for word in procw {
            let mut rel = vec![];
            let mut urel = vec![];
            relative_representation(word.chars(), &mut rel, &mut rel_map);
            buf.clear();
            relative_representation(rel.iter().copied(), &mut urel, &mut buf);
            self.words.push(Word { rel, urel });
        }
    }

    fn load_dict<E>(
        &mut self,
        i: impl Iterator<Item = Result<String, E>>,
    ) -> Result<(), E> {
        let lengths: HashSet<_> =
            self.orig_words.iter().map(|a| a.len()).unique().collect();

        self.dict
            .extend(self.words.iter().map(|r| (r.urel.clone(), vec![])));

        let mut buf = HashMap::new();
        for s in i {
            let s = s?;
            let s = s.trim();
            if !lengths.contains(&s.len()) {
                continue;
            }

            let mut rel = vec![];
            buf.clear();
            relative_representation(s.chars(), &mut rel, &mut buf);
            let rlen = rel.len();
            self.dict.entry(rel).and_modify(|a| {
                if rlen == s.len() {
                    a.push(s.to_string())
                }
            });
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

fn relative_representation<T: Hash + Eq>(
    s: impl IntoIterator<Item = T>,
    res: &mut Vec<usize>,
    map: &mut HashMap<T, usize>,
) {
    res.clear();
    res.extend(s.into_iter().map(|c| {
        let len = map.len();
        *map.entry(c).or_insert(len)
    }));
}
