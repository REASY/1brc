use crate::{byte_to_string_unsafe, StateF, StateI};

/// Open addressing table
pub struct Table<const MAX_SIZE: usize> {
    // inner: [Option<(String, StateI64)>; MAX_SIZE], // stack allocation
    inner: Vec<Option<(String, StateI)>>,
}

impl<const MAX_SIZE: usize> Table<MAX_SIZE> {
    pub fn new() -> Table<MAX_SIZE> {
        Table {
            // inner: [(); MAX_SIZE].map(|()| None), // stack allocation
            inner: (0..MAX_SIZE).map(|_| None).collect(),
        }
    }

    #[inline]
    fn find_slot(&mut self, key: &[u8], hash: u64) -> &mut Option<(String, StateI)> {
        let mut iter_idx: usize = 0;
        let len = self.inner.len();
        let slot_idx = loop {
            // Linear probing
            let idx_mod: usize = (hash as usize + iter_idx) % len;
            match &self.inner[idx_mod] {
                Some((k, _)) if k.as_bytes().eq(key) => break idx_mod,
                None => break idx_mod,
                _ => {
                    iter_idx += 1;

                    assert!(
                        iter_idx <= len,
                        "find_slot called without a matching key and full storage!"
                    );
                }
            }
        };
        &mut self.inner[slot_idx]
    }

    #[inline]
    pub fn insert_or_update(&mut self, key: &[u8], hash: u64, value: i16) {
        let slot = self.find_slot(key, hash);
        if slot.is_none() {
            let _ = std::mem::replace(
                slot,
                Some((byte_to_string_unsafe(key).to_string(), StateI::new(value))),
            );
        } else {
            let (_, state) = slot.as_mut().unwrap();
            state.update(value)
        }
    }

    pub fn to_result(&self) -> Vec<(String, StateF)> {
        let mut result: Vec<(String, StateF)> = Vec::with_capacity(MAX_SIZE);
        for item in &self.inner {
            match item {
                None => {}
                Some((k, v)) => {
                    result.push((k.clone(), v.to_f64()));
                }
            }
        }
        result
    }
}
#[allow(unused)]
/// Slower than Table because Table has data locality
pub struct KeyValueTable<const MAX_SIZE: usize> {
    keys: Vec<Option<String>>,
    values: Vec<StateI>,
}

#[allow(unused)]
impl<const MAX_SIZE: usize> KeyValueTable<MAX_SIZE> {
    pub fn new() -> KeyValueTable<MAX_SIZE> {
        KeyValueTable {
            keys: (0..MAX_SIZE).map(|_| None).collect(),
            values: (0..MAX_SIZE).map(|_| StateI::default()).collect(),
        }
    }

    #[inline]
    fn find_slot(&mut self, hash: u64, key: &[u8]) -> usize {
        let mut iter_idx: usize = 0;
        let len: usize = self.keys.len();
        let slot_idx = loop {
            // Linear probing
            let idx_mod: usize = (hash as usize + iter_idx) % len;
            match &self.keys[idx_mod] {
                Some(k) if k.as_bytes().eq(key) => break idx_mod,
                None => break idx_mod,
                _ => {
                    iter_idx += 1;
                    assert!(
                        iter_idx <= len,
                        "find_slot called without a matching key and full storage!"
                    );
                }
            }
        };
        slot_idx
    }

    #[inline]
    pub fn insert_or_update(&mut self, key: &[u8], hash: u64, value: i16) {
        let slot_idx = self.find_slot(hash, key);
        let slot = &mut self.keys[slot_idx];
        if slot.is_none() {
            let new = Some(byte_to_string_unsafe(key).to_string());
            let _ = std::mem::replace(slot, new);
        } else {
            let state: &mut StateI = &mut self.values[slot_idx];
            state.update(value)
        }
    }

    pub fn to_result(&self) -> Vec<(String, StateF)> {
        let mut result: Vec<(String, StateF)> = Vec::with_capacity(MAX_SIZE);
        for i in 0..self.keys.len() {
            match &self.keys[i] {
                None => {}
                Some(k) => {
                    result.push((k.clone(), self.values[i].to_f64()));
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_or_update() {
        let mut t: Table<10000> = Table::new();

        let k0 = "hello".as_bytes();
        let k1 = "world".as_bytes();
        let h0: u64 = 1;

        t.insert_or_update(k0, h0, 1);
        let (fk0, fv0) = t.find_slot(k0, h0).as_ref().unwrap();
        assert_eq!("hello".to_string(), *fk0);
        assert_eq!(1, fv0.max);
        assert_eq!(1, fv0.min);
        assert_eq!(1, fv0.count);
        assert_eq!(1, fv0.sum);

        t.insert_or_update(k0, h0, 1);
        let (fk0, fv0) = t.find_slot(k0, h0).as_ref().unwrap();
        assert_eq!("hello".to_string(), *fk0);
        assert_eq!(1, fv0.max);
        assert_eq!(1, fv0.min);
        assert_eq!(2, fv0.count);
        assert_eq!(2, fv0.sum);

        t.insert_or_update(k0, h0, 2);
        let (fk0, fv0) = t.find_slot(k0, h0).as_ref().unwrap();
        assert_eq!("hello".to_string(), *fk0);
        assert_eq!(2, fv0.max);
        assert_eq!(1, fv0.min);
        assert_eq!(3, fv0.count);
        assert_eq!(4, fv0.sum);

        // Same hash but different value should give none
        let r = t.find_slot(k1, h0).as_ref();
        assert!(r.is_none());

        t.insert_or_update(k1, h0, 5);
        let (fk1, fv1) = t.find_slot(k1, h0).as_ref().unwrap();
        assert_eq!("world".to_string(), *fk1);
        assert_eq!(5, fv1.max);
        assert_eq!(5, fv1.min);
        assert_eq!(1, fv1.count);
        assert_eq!(5, fv1.sum);
    }
}
