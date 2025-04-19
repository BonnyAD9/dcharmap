use std::{collections::HashMap, hash::Hash};

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
    s: String,
    rel: Vec<usize>,
    urel: Vec<usize>,
    dependencies: u64,
}

impl FcmData {
    pub fn find_char_map(&self) -> Vec<WordTree> {
        if self.words.is_empty() {
            return vec![];
        }

        let mut rel = vec![];
        let mut rmap = HashMap::new();
        let mut stack = vec![];

        let Some(opts) = self.dict.get(&self.words[0].urel) else {
            return vec![];
        };

        let mut ret = None;
        let mut ropt = vec![];

        stack.push((vec![], rmap.len(), opts.iter()));
        'outer: while let Some((mut res, len, mut opts)) = stack.pop() {
            if let Some(next) = ret.take() {
                res.push(WordTree {
                    word: ropt.pop().unwrap(),
                    next,
                });
            } else if ropt.len() > stack.len() {
                ropt.pop();
            }

            while let Some(opt) = opts.next() {
                if rmap.len() > len {
                    if len == 0 {
                        rmap.clear();
                    } else {
                        rmap.retain(|_, e| *e < len);
                    }
                }

                relative_representation(opt.chars(), &mut rel, &mut rmap);
                if rel == self.words[stack.len()].rel {
                    stack.push((res, len, opts));
                    ropt.push(opt);

                    let Some(opts) = self
                        .words
                        .get(stack.len())
                        .and_then(|word| self.dict.get(&word.urel))
                    else {
                        ret = Some(vec![]);
                        continue 'outer;
                    };
                    ret = None;
                    stack.push((vec![], rmap.len(), opts.iter()));
                    continue 'outer;
                }
            }

            ret = (!res.is_empty()).then_some(res);
        }

        ret.unwrap_or_default()
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
        res.sort_words();

        Ok(res)
    }

    fn set_words(&mut self, words: Vec<String>) {
        self.orig_words = words;
        let procw = self
            .orig_words
            .iter()
            .map(|w| w.as_str())
            .unique()
            .collect_vec();

        let mut rel_map = HashMap::new();
        let mut buf = HashMap::new();
        for word in procw {
            let mut rel = vec![];
            let mut urel = vec![];
            relative_representation(word.chars(), &mut rel, &mut rel_map);
            buf.clear();
            relative_representation(rel.iter().copied(), &mut urel, &mut buf);
            self.words.push(Word {
                rel,
                urel,
                s: word.to_string(),
                dependencies: 0,
            });
        }
    }

    fn load_dict<E>(
        &mut self,
        i: impl Iterator<Item = Result<String, E>>,
    ) -> Result<(), E> {
        self.dict
            .extend(self.words.iter().map(|r| (r.urel.clone(), vec![])));

        let mut buf = HashMap::new();
        for s in i {
            let s = s?;
            let s = s.trim();

            let mut rel = vec![];
            buf.clear();
            relative_representation(s.chars(), &mut rel, &mut buf);
            self.dict.entry(rel).and_modify(|a| a.push(s.to_string()));
        }

        Ok(())
    }

    fn sort_words(&mut self) {
        let mut deps = vec![];
        for (i0, word) in self.words.iter().enumerate() {
            let mut dep = 0;
            for (i, w) in self.words.iter().enumerate() {
                if i0 == i {
                    continue;
                }

                for (i, c) in word.rel.iter().enumerate() {
                    if w.rel.contains(c) {
                        dep |= 1 << i;
                    }
                }
            }
            deps.push(dep);
        }

        for (w, d) in self.words.iter_mut().zip(deps) {
            w.dependencies = d;
        }

        // Move the most limiting word as first.
        let Some((i, _)) = self
            .words
            .iter()
            .enumerate()
            .max_by_key(|(_, w)| (w.dependencies.count_ones(), w.s.len()))
        else {
            return;
        };
        self.words.swap(0, i);

        // Sort the words from least freedom to most freedom.
        self.words[1..].sort_unstable_by_key(|w| {
            (w.freedom(), self.dict.get(&w.urel).unwrap().len())
        });

        self.word_map = self
            .orig_words
            .iter()
            .map(|w| {
                for (i, pw) in self.words.iter().enumerate() {
                    if &pw.s == w {
                        return i;
                    }
                }
                unreachable!()
            })
            .collect();

        let mut rel_map = HashMap::new();
        for word in &mut self.words {
            relative_representation(
                word.s.chars(),
                &mut word.rel,
                &mut rel_map,
            );
        }
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

impl Word {
    fn freedom(&self) -> usize {
        self.urel.len() - self.dependencies.count_ones() as usize
    }
}
