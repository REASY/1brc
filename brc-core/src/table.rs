use crate::{byte_to_string_unsafe, Holder, StateF64, StateI64};
use rustc_hash::FxHasher;
use std::hash::Hasher;

pub struct Table<const MAX_SIZE: usize> {
    inner: [Option<(String, StateI64)>; MAX_SIZE],
    // inner: Vec<Option<(&'a [u8], StateI64)>>,
}

enum Offset {
    Exists(u32),
    ToInsert(u32),
    Overflow,
}

impl<const MAX_SIZE: usize> Table<MAX_SIZE> {
    pub fn new() -> Table<MAX_SIZE> {
        Table {
            inner: [(); MAX_SIZE].map(|()| None),
            // inner: (0..MAX_SIZE).map(|_| None).collect(),
        }
    }

    #[inline]
    fn find_slot(&mut self, hash: u64, key: &[u8]) -> &mut Option<(String, StateI64)> {
        let mut iter_idx: usize = 0;
        let len = self.inner.len();

        let start_idx = hash as usize;
        let slot_idx = loop {
            let idx_mod: usize = (start_idx + iter_idx) % len;
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
        // let s = byte_to_string_unsafe(station_name_bytes);
        let mut slot = self.find_slot(hash, station_name_bytes);
        if slot.is_none() {
            // let name = holder.store(station_name_bytes);
            let _ = std::mem::replace(
                slot,
                Some((
                    byte_to_string_unsafe(station_name_bytes).to_string(),
                    StateI64::new(value),
                )),
            );
        } else {
            let mut state = &mut slot.as_mut().unwrap().1;
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
