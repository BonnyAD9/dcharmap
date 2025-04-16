use std::collections::{BTreeSet, HashMap};

pub fn find_char_map(
    words: &[String],
    dict: &HashMap<usize, BTreeSet<String>>,
) -> Vec<Vec<String>> {
    let mut solutions =
        find_char_map_inner(words, dict, HashMap::new(), HashMap::new());
    for s in &mut solutions {
        s.reverse();
    }
    solutions
}

pub fn find_char_map_inner(
    words: &[String],
    dict: &HashMap<usize, BTreeSet<String>>,
    mut emap: HashMap<char, usize>,
    mut umap: HashMap<char, usize>,
) -> Vec<Vec<String>> {
    if words.is_empty() {
        return vec![vec![]];
    }

    let mut erep = vec![];
    let mut urep = vec![];

    let word = &words[0];

    relative_representation(word, &mut erep, &mut emap);

    let Some(opts) = dict.get(&word.len()) else {
        return vec![];
    };

    let len = umap.len();

    let mut res = vec![];

    for opt in opts {
        relative_representation(opt, &mut urep, &mut umap);
        if urep == erep {
            for mut s in find_char_map_inner(
                &words[1..],
                dict,
                emap.clone(),
                umap.clone(),
            ) {
                s.push(opt.to_string());
                res.push(s);
            }
        }
        umap.retain(|_, e| *e < len);
    }

    res
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
