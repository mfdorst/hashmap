use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::mem;

const INITIAL_NUM_BUCKETS: usize = 1;

pub struct HashMap<K, V> {
    buckets: Vec<Vec<(K, V)>>,
    items: usize,
}

impl<K, V> HashMap<K, V> {
    pub fn new() -> Self {
        HashMap {
            buckets: Vec::new(),
            items: 0,
        }
    }
}

impl<K, V> HashMap<K, V>
where
    K: Hash + Eq,
{
    fn resize(&mut self) {
        let target_size = match self.buckets.len() {
            0 => INITIAL_NUM_BUCKETS,
            n => n * 2,
        };

        let mut new_buckets = Vec::with_capacity(target_size);
        new_buckets.extend((0..target_size).map(|_| vec![]));

        for (key, value) in self.buckets.iter_mut().flat_map(|bucket| bucket.drain(..)) {
            let index = Self::hash_index(&key, new_buckets.len());
            new_buckets[index].push((key, value));
        }

        self.buckets = new_buckets;
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.items += 1;
        if self.items > self.buckets.len() * 3 / 4 {
            self.resize();
        }
        let index = Self::hash_index(&key, self.buckets.len());
        let bucket = &mut self.buckets[index];
        for &mut (ref k, ref mut v) in bucket.iter_mut() {
            if k == &key {
                return Some(mem::replace(v, value));
            }
        }
        bucket.push((key, value));
        None
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.buckets[Self::hash_index(key, self.buckets.len())]
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let index = Self::hash_index(key, self.buckets.len());
        let bucket = &mut self.buckets[index];
        let i = bucket.iter().position(|(k, _)| k == key)?;
        let (_, v) = bucket.swap_remove(i);
        self.items -= 1;
        Some(v)
    }

    pub fn len(&self) -> usize {
        self.items
    }

    pub fn is_empty(&self) -> bool {
        self.items == 0
    }

    fn hash_index(key: &K, table_size: usize) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() % table_size as u64) as usize
    }
}

pub struct Iter<'a, K, V> {
    map: &'a HashMap<K, V>,
    bucket_index: usize,
    elem_index: usize,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.map.buckets.get(self.bucket_index) {
                Some(bucket) => match bucket.get(self.elem_index) {
                    Some((k, v)) => {
                        self.elem_index += 1;
                        break Some((&k, &v));
                    }
                    None => {
                        self.bucket_index += 1;
                        self.elem_index = 0;
                        continue;
                    }
                },
                None => break None,
            }
        }
    }
}

impl<'a, K, V> Iter<'a, K, V> {
    fn new(map: &'a HashMap<K, V>) -> Self {
        Iter {
            map,
            bucket_index: 0,
            elem_index: 0,
        }
    }
}

impl<'a, K, V> IntoIterator for &'a HashMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;
    fn into_iter(self) -> Iter<'a, K, V> {
        Iter::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_operations() {
        let mut map = HashMap::new();
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
        map.insert("foo", 42);
        assert_eq!(map.len(), 1);
        assert!(!map.is_empty());
        assert!(map.contains_key(&"foo"));
        assert_eq!(map.get(&"foo"), Some(&42));
        assert_eq!(map.remove(&"foo"), Some(42));
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
        assert_eq!(map.get(&"foo"), None);
    }

    #[test]
    fn iter() {
        let mut map = HashMap::new();
        map.insert("foo", 42);
        map.insert("bar", 45);
        map.insert("baz", 50);
        map.insert("quox", 55);
        for (&k, &v) in &map {
            match k {
                "foo" => assert_eq!(v, 42),
                "bar" => assert_eq!(v, 45),
                "baz" => assert_eq!(v, 50),
                "quox" => assert_eq!(v, 55),
                _ => unreachable!(),
            }
        }
        assert_eq!((&map).into_iter().count(), 4);
    }
}
