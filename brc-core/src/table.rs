use crate::{byte_to_string_unsafe, StateF64, StateI64};

/// Open addressing table
pub struct Table<const MAX_SIZE: usize> {
    // inner: [Option<(String, StateI64)>; MAX_SIZE], // stack allocation
    inner: Vec<Option<(String, StateI64)>>,
}

enum Offset {
    Exists(u32),
    ToInsert(u32),
    Overflow,
}

impl<const MAX_SIZE: usize> Table<MAX_SIZE> {
    pub fn new() -> Table<MAX_SIZE> {
        Table {
            // inner: [(); MAX_SIZE].map(|()| None), // stack allocation
            inner: (0..MAX_SIZE).map(|_| None).collect(),
        }
    }

    #[inline]
    fn find_slot(&mut self, hash: u64, key: &[u8]) -> &mut Option<(String, StateI64)> {
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
    pub fn insert_or_update(&mut self, station_name_bytes: &[u8], hash: u64, value: i16) {
        let slot = self.find_slot(hash, station_name_bytes);
        if slot.is_none() {
            let _ = std::mem::replace(
                slot,
                Some((
                    byte_to_string_unsafe(station_name_bytes).to_string(),
                    StateI64::new(value),
                )),
            );
        } else {
            let (_, state) = slot.as_mut().unwrap();
            state.update(value)
        }
    }

    pub fn to_result(&self) -> Vec<(String, StateF64)> {
        let mut result: Vec<(String, StateF64)> = Vec::with_capacity(MAX_SIZE);
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
/// Slower than Table because Table has data locality
pub struct KeyValueTable<const MAX_SIZE: usize> {
    keys: Vec<Option<String>>,
    values: Vec<StateI64>,
}

impl<const MAX_SIZE: usize> KeyValueTable<MAX_SIZE> {
    pub fn new() -> KeyValueTable<MAX_SIZE> {
        KeyValueTable {
            keys: (0..MAX_SIZE).map(|_| None).collect(),
            values: (0..MAX_SIZE).map(|_| StateI64::default()).collect(),
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
            let state: &mut StateI64 = &mut self.values[slot_idx];
            state.update(value)
        }
    }

    pub fn to_result(&self) -> Vec<(String, StateF64)> {
        let mut result: Vec<(String, StateF64)> = Vec::with_capacity(MAX_SIZE);
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
