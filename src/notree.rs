#[derive(Debug)]
pub struct Notree<K: Copy + Ord, V> {
    nodes: Vec<(K, Notree<K, V>)>,
    leafs: Vec<V>,
}

impl<K: Copy + Ord, V> Notree<K, V> {
    pub fn add(&mut self, key: &[K], value: V) {
        if key.is_empty() {
            self.leafs.push(value);
            return;
        }

        //self.nodes.entry(key[0]).or_default().add(&key[1..], value);
        match self.nodes.binary_search_by_key(&key[0], |a| a.0) {
            Ok(idx) => self.nodes[idx].1.add(&key[1..], value),
            Err(idx) => {
                self.nodes.insert(idx, (key[0], Notree::default()));
                self.nodes[idx].1.add(&key[1..], value)
            }
        }
    }

    pub fn no_values_to<'a>(
        &'a self,
        mut key: impl Iterator<Item = K> + Clone,
        res: &mut Vec<&'a V>,
    ) {
        res.extend(self.leafs.iter());
        let mut ki = key.next();

        for (nk, nv) in &self.nodes {
            while matches!(ki, Some(i) if *nk > i) {
                ki = key.next();
            }

            if ki.is_none() || matches!(ki, Some(i) if *nk < i) {
                nv.no_values_to(key.clone(), res);
                continue;
            }

            ki = key.next();
        }
    }

    pub fn no_values(&self, key: impl Iterator<Item = K> + Clone) -> Vec<&V> {
        let mut res = vec![];
        self.no_values_to(key, &mut res);
        res
    }
}

impl<K: Copy + Ord, V> Default for Notree<K, V> {
    fn default() -> Self {
        Self {
            nodes: Default::default(),
            leafs: Default::default(),
        }
    }
}
