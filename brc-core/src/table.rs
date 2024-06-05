use crate::{byte_to_string_unsafe, Holder, StateF64, StateI64};

pub struct Table<'a, const MAX_SIZE: usize, const COLLISION_KEY_SIZE: usize> {
    inner: [[(&'a [u8], StateI64); COLLISION_KEY_SIZE]; MAX_SIZE],
}

enum Offset {
    Exists(u32),
    ToInsert(u32),
    Overflow,
}

impl<'a, const MAX_SIZE: usize, const COLLISION_KEY_SIZE: usize>
    Table<'a, MAX_SIZE, COLLISION_KEY_SIZE>
{
    pub fn new() -> Table<'a, MAX_SIZE, COLLISION_KEY_SIZE> {
        Table {
            inner: [(); MAX_SIZE].map(|()| [(); COLLISION_KEY_SIZE].map(|()| Default::default())),
        }
    }

    #[inline]
    fn find_offset(&mut self, table_idx: usize, station_name_bytes: &[u8]) -> Offset {
        let bucket = &self.inner[table_idx];
        for i in 0..COLLISION_KEY_SIZE {
            let (k, _) = bucket[i];
            if k.is_empty() {
                return Offset::ToInsert(i as u32);
            }
            if k == station_name_bytes {
                return Offset::Exists(i as u32);
            }
        }
        Offset::Overflow
    }

    #[inline]
    pub fn insert_or_update(
        &mut self,
        holder: &mut Holder<'a>,
        station_name_bytes: &[u8],
        hash: u64,
        value: i32,
    ) {
        let table_idx = (hash as usize) % MAX_SIZE;
        // let s = byte_to_string_unsafe(station_name_bytes);
        match self.find_offset(table_idx, station_name_bytes) {
            Offset::Exists(offset) => {
                self.inner[table_idx][offset as usize]
                    .1
                    .update(value as i64);
            }
            Offset::ToInsert(offset) => {
                let name = holder.store(station_name_bytes);
                self.inner[table_idx][offset as usize] = (name, StateI64::new(value as i64));
                // self.hs.insert(s.to_string());
            }
            Offset::Overflow => {
                panic!(
                    "table_idx: {}, COLLISION_KEY_SIZE: {}",
                    table_idx, COLLISION_KEY_SIZE
                );
            }
        }
    }

    pub fn to_result(&self) -> Vec<(String, StateF64)> {
        let mut result: Vec<(String, StateF64)> = Vec::with_capacity(MAX_SIZE);
        for item in &self.inner {
            for (k, v) in item {
                if (*k).is_empty() {
                    break;
                }
                result.push((byte_to_string_unsafe(*k).to_string(), v.to_f64()))
            }
        }
        result
    }
}
