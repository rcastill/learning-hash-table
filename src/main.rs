use std::borrow::Cow;

const DEFAULT_HASH2ST_SIZE: usize = 256;

fn default_hash(s: &str, len: usize) -> usize {
    s.chars()
        .map(|c| {
            let v: u64 = c.into();
            v as usize
        })
        .sum::<usize>()
        % len
}

struct HashItem<T> {
    k: Cow<'static, str>,
    v: T,
}

// Each hashnode has an inner vector, since we are
// using Closed Addressing
type HashNode<T> = Option<Vec<HashItem<T>>>;

struct HashS2T<T> {
    items: Vec<HashNode<T>>,
    stat_collisions: usize,
}

impl<T> Default for HashS2T<T> {
    fn default() -> Self {
        let mut items = Vec::with_capacity(DEFAULT_HASH2ST_SIZE);
        // vec![None; ...] requires Node: Clone
        for _ in 0..DEFAULT_HASH2ST_SIZE {
            items.push(None);
        }
        Self {
            items,
            stat_collisions: 0,
        }
    }
}

impl<T> HashS2T<T> {
    fn insert(&mut self, k: &str, v: T) {
        if let Some(item) = self.get_item_mut(k) {
            item.v = v;
            return;
        }
        let i = default_hash(k, self.items.len());
        let item = HashItem {
            k: k.to_string().into(),
            v,
        };
        match &mut self.items[i] {
            Some(items) => {
                self.stat_collisions += 1;
                items.push(item);
            }
            None => self.items[i] = Some(vec![item]),
        }
    }

    fn get_item(&self, k: &str) -> Option<&HashItem<T>> {
        if self.items.is_empty() {
            return None;
        }
        let i = default_hash(k, self.items.len());
        let node = &self.items[i];
        node.as_ref()
            .and_then(|items| items.iter().find(|item| item.k == k))
    }

    fn get(&self, k: &str) -> Option<&T> {
        self.get_item(k).map(|HashItem { v, .. }| v)
    }

    fn get_item_mut(&mut self, k: &str) -> Option<&mut HashItem<T>> {
        if self.items.is_empty() {
            return None;
        }
        let i = default_hash(k, self.items.len());
        let node = &mut self.items[i];
        node.as_mut()
            .and_then(|items| items.iter_mut().find(|item| item.k == k))
    }

    #[allow(dead_code)]
    fn get_mut(&mut self, k: &str) -> Option<&mut T> {
        self.get_item_mut(k).map(|HashItem { v, .. }| v)
    }

    fn into_iter(self) -> impl Iterator<Item = HashItem<T>> {
        self.items.into_iter().filter_map(|node| node).flatten()
    }

    fn iter(&self) -> impl Iterator<Item = &HashItem<T>> {
        self.items.iter().filter_map(|node| node.as_ref()).flatten()
    }
}

impl<T> IntoIterator for HashS2T<T>
where
    T: 'static,
{
    type Item = HashItem<T>;

    // TODO: static type -- it is a composed iterator -- too much work
    type IntoIter = Box<dyn Iterator<Item = HashItem<T>>>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(HashS2T::into_iter(self))
    }
}

impl<'a, T> IntoIterator for &'a HashS2T<T> {
    type Item = &'a HashItem<T>;

    // TODO: static type -- it is a composed iterator -- too much work
    type IntoIter = Box<dyn Iterator<Item = &'a HashItem<T>> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.iter())
    }
}

fn main() {
    let mut h = HashS2T::default();
    h.insert("Woffo", 1);
    h.insert("Gato", 2);
    for HashItem { k, v } in &h {
        eprintln!("{k}\t: {v}");
    }

    eprintln!();
    let woffo_hash = default_hash("Woffo", h.items.len());
    let gato_hash = default_hash("Gato", h.items.len());
    eprintln!("hash(Woffo)\t: {woffo_hash}");
    eprintln!("hash(Gato)\t: {gato_hash}");

    eprintln!();
    let woffo = *h.get("Woffo").unwrap();
    let gato = *h.get("Gato").unwrap();
    eprintln!("get(Woffo)\t: {woffo}");
    eprintln!("get(Gato)\t: {gato}");
}

#[cfg(test)]
mod test {
    use std::fmt::Debug;

    use super::*;

    fn expected_items<T>(h: &HashS2T<T>, expected: &[(&str, T)])
    where
        T: PartialOrd + Clone + Debug,
    {
        // values must exist
        let mut items: Vec<_> = h.iter().collect();
        items.sort_by(|HashItem { v: v1, .. }, HashItem { v: v2, .. }| v1.partial_cmp(v2).unwrap());
        assert_eq!(
            items
                .iter()
                .map(|HashItem { k, v }| (k.as_ref(), v.clone()))
                .collect::<Vec<_>>(),
            expected
        );
    }

    #[test]
    fn insert() {
        let mut h = HashS2T::default();
        h.insert("a", 1);
        h.insert("b", 2);
        h.insert("c", 2);
        h.insert("c", 3);

        assert_eq!(h.items.len(), DEFAULT_HASH2ST_SIZE);

        // values must exist
        expected_items(&h, &[("a", 1), ("b", 2), ("c", 3)]);
    }

    #[test]
    fn stress() {
        let mut h = HashS2T::default();
        for key_i in 0..5000 {
            let key = format!("key_{key_i}");
            let val = key_i + 42;
            h.insert(&key, val);
            // insert twice
            h.insert(&key, val);
            assert_eq!(*h.get(&key).unwrap(), val)
        }
    }

    #[test]
    fn get() {
        let mut h = HashS2T::default();
        // works with no contents
        let _opt = h.get("gg");

        h.insert("a", 1);
        h.insert("R", 42);
        h.insert("c", 3);

        assert_eq!(h.get("R"), Some(&42));
        assert_eq!(h.get("Q"), None);
    }
}
