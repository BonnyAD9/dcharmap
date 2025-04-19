use std::collections::HashMap;

pub struct WordTree<'a> {
    pub word: &'a str,
    pub next: Vec<WordTree<'a>>,
}

pub fn find_char_map<'a>(
    words: &[String],
    dict: &'a HashMap<usize, Vec<String>>,
) -> Vec<WordTree<'a>> {
    find_char_map_inner(words, dict, HashMap::new(), HashMap::new())
        .unwrap_or_default()
}

pub fn find_char_map_inner<'a>(
    words: &[String],
    dict: &'a HashMap<usize, Vec<String>>,
    mut emap: HashMap<char, usize>,
    mut umap: HashMap<char, usize>,
) -> Option<Vec<WordTree<'a>>> {
    if words.is_empty() {
        return Some(vec![]);
    }

    let mut erep = vec![];
    let mut urep = vec![];

    let word = &words[0];

    relative_representation(word, &mut erep, &mut emap);

    let opts = dict.get(&word.len())?;

    let len = umap.len();

    let mut res = vec![];

    for opt in opts {
        relative_representation(opt, &mut urep, &mut umap);
        if urep == erep {
            let branches = find_char_map_inner(
                &words[1..],
                dict,
                emap.clone(),
                umap.clone(),
            );
            if let Some(next) = branches {
                let tree = WordTree { word: opt, next };
                res.push(tree)
            }
        }
        umap.retain(|_, e| *e < len);
    }

    (!res.is_empty()).then_some(res)
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
