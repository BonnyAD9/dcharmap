use std::{
    collections::{BTreeSet, HashMap},
    hash::Hash,
};

use itertools::Itertools;

use crate::notree::Notree;

pub struct WordTree<'a> {
    pub word: &'a str,
    pub next: Vec<WordTree<'a>>,
}

pub struct FcmData {
    orig_words: Vec<String>,
    word_map: Vec<usize>,
    dict: HashMap<Vec<usize>, HashMap<String, Notree<char, String>>>,
    words: Vec<Word>,
}

struct Word {
    s: String,
    rel: Vec<usize>,
    urel: Vec<usize>,
    dependencies: u64,
    back_dep: Vec<usize>,
}

impl FcmData {
    pub fn find_char_map(&self) -> Vec<WordTree> {
        if self.words.is_empty() {
            return vec![];
        }

        let mut rel = vec![];
        let mut rmap = HashMap::new();
        let mut irmap = vec![];
        let mut stack = vec![];
        let mut sbuf = String::new();
        let mut sorted = BTreeSet::<char>::new();

        let Some(opts) =
            self.dict.get(&self.words[0].urel).and_then(|o| o.get(""))
        else {
            return vec![];
        };

        let mut ret = None;
        let mut ropt = vec![];

        stack.push((
            vec![],
            rmap.len(),
            opts.no_values([].iter().copied()).into_iter(),
        ));
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
                    irmap.splice(len.., []);
                }

                relative_representation_i(
                    opt.chars(),
                    &mut rel,
                    &mut rmap,
                    &mut irmap,
                );
                if rel == self.words[stack.len()].rel {
                    stack.push((res, len, opts));
                    ropt.push(opt);

                    let Some(word) = self.words.get(stack.len()) else {
                        ret = Some(vec![]);
                        continue 'outer;
                    };
                    let Some(opts) = self.dict.get(&word.urel).and_then(|o| {
                        sbuf.clear();
                        fixed_hash_map(
                            &word.back_dep,
                            &word.rel,
                            &irmap,
                            &mut sbuf,
                        );
                        o.get(&sbuf)
                    }) else {
                        ret = None;
                        continue 'outer;
                    };
                    ret = None;

                    sorted.clear();
                    sorted.extend(
                        irmap[..len]
                            .iter()
                            .enumerate()
                            .filter(|(i, _)| !word.rel.contains(i))
                            .map(|(_, v)| v),
                    );

                    stack.push((
                        vec![],
                        rmap.len(),
                        opts.no_values(sorted.iter().copied()).into_iter(),
                    ));
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
        let dict = res.load_dict(di)?;

        res.resolve_dependencies();
        // this is optional optimization that requires `resolve_dependencies`
        res.sort_words(&dict);

        res.resolve_back_dependencies();
        res.remap_dict(dict);
        //res.specialize_back_deps();

        res.create_word_map();

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
                back_dep: vec![],
            });
        }
    }

    fn load_dict<E>(
        &mut self,
        i: impl Iterator<Item = Result<String, E>>,
    ) -> Result<HashMap<Vec<usize>, Vec<String>>, E> {
        let mut res: HashMap<_, _> = self
            .words
            .iter()
            .map(|r| (r.urel.clone(), vec![]))
            .collect();

        let mut buf = HashMap::new();
        for s in i {
            let s = s?;
            let s = s.trim();

            let mut rel = vec![];
            buf.clear();
            relative_representation(s.chars(), &mut rel, &mut buf);
            res.entry(rel).and_modify(|a| a.push(s.to_string()));
        }

        Ok(res)
    }

    fn resolve_dependencies(&mut self) {
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
    }

    fn resolve_back_dependencies(&mut self) {
        let mut deps = vec![];
        for (i0, word) in self.words.iter().enumerate() {
            let mut dep = vec![];
            for w in self.words[..i0].iter() {
                for (i, c) in word.rel.iter().enumerate() {
                    if (w.dependencies & (1 << i)) != 0 && w.rel.contains(c) {
                        dep.push(i);
                    }
                }
            }
            deps.push(dep);
        }

        for (w, d) in self.words.iter_mut().zip(deps) {
            w.back_dep = d;
        }
    }

    fn remap_dict(&mut self, mut dict: HashMap<Vec<usize>, Vec<String>>) {
        let mut buf = String::new();
        for w in &self.words {
            let Some((k, s)) = dict.remove_entry(&w.urel) else {
                continue;
            };

            let mut v: HashMap<String, Notree<char, String>> = HashMap::new();

            for s in s {
                buf.clear();
                fixed_hash(&w.back_dep, &s, &mut buf);
                let mut key: Vec<_> = s.chars().unique().collect();
                key.sort_unstable();
                v.entry(buf.clone()).or_default().add(&key, s);
            }

            self.dict.insert(k, v);
        }
    }

    fn sort_words(&mut self, dict: &HashMap<Vec<usize>, Vec<String>>) {
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
            (w.freedom(), dict.get(&w.urel).unwrap().len())
        });

        let mut rel_map = HashMap::new();
        for word in &mut self.words {
            relative_representation(
                word.s.chars(),
                &mut word.rel,
                &mut rel_map,
            );
        }
    }

    fn create_word_map(&mut self) {
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

fn relative_representation_i<T: Hash + Eq + Copy>(
    s: impl IntoIterator<Item = T>,
    res: &mut Vec<usize>,
    map: &mut HashMap<T, usize>,
    imap: &mut Vec<T>,
) {
    res.clear();
    res.extend(s.into_iter().map(|c| {
        let len = map.len();
        *map.entry(c).or_insert_with(|| {
            imap.push(c);
            len
        })
    }));
}

fn fixed_hash(dep: &[usize], w: &str, res: &mut String) {
    let mut ci = w.char_indices();
    let mut last = 0;
    for d in dep {
        let c = ci.nth(*d - last).unwrap().1;
        last = *d + 1;
        res.push(c);
    }
}

fn fixed_hash_map(
    dep: &[usize],
    rel: &[usize],
    map: &[char],
    res: &mut String,
) {
    for i in dep {
        res.push(map[rel[*i]]);
    }
}

impl Word {
    fn freedom(&self) -> usize {
        self.urel.len() - self.dependencies.count_ones() as usize
    }
}
