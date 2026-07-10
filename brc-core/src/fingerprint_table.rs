use std::simd::cmp::SimdOrd;
use std::simd::{Simd, i64x8};

use crate::{StateF, StateI};

#[inline(always)]
fn fingerprint(name: &[u8], padded_name: &[u8]) -> u64 {
    const MASK: [u64; 9] = [
        0,
        0xff,
        0xffff,
        0xffffff,
        0xffffffff,
        0xffffffffff,
        0xffffffffffff,
        0xffffffffffffff,
        u64::MAX,
    ];
    let len = name.len();
    let mask = MASK[len.min(8)];
    let first = u64::from_le_bytes(padded_name[..8].try_into().unwrap()) & mask;
    let last_offset = len.saturating_sub(8);
    let last = u64::from_le_bytes(
        padded_name[last_offset..last_offset + 8]
            .try_into()
            .unwrap(),
    ) & mask;
    first ^ last.rotate_left(29) ^ (len as u64).wrapping_mul(0x9e3779b97f4a7c15)
}

struct FingerprintSlot {
    hash: u64,
    state: StateI,
    name_id: u16,
}

impl Default for FingerprintSlot {
    fn default() -> Self {
        Self {
            hash: 0,
            state: StateI::default(),
            name_id: 0,
        }
    }
}

/// A compact table for the challenge's small station cardinality. Hash equality
/// is treated as key equality, deliberately trading general-purpose collision
/// handling for avoiding one variable-length name comparison on every record.
pub(crate) struct FingerprintTable<const SIZE: usize> {
    slots: Vec<FingerprintSlot>,
    names: Vec<String>,
}

impl<const SIZE: usize> FingerprintTable<SIZE> {
    pub(crate) fn new() -> Self {
        assert!(SIZE.is_power_of_two());
        Self {
            slots: (0..SIZE).map(|_| FingerprintSlot::default()).collect(),
            names: Vec::with_capacity(512),
        }
    }

    #[inline(always)]
    pub(crate) fn insert_or_update(&mut self, name: &[u8], padded_name: &[u8], value: i16) {
        let raw_fingerprint = fingerprint(name, padded_name);
        let hash = if raw_fingerprint == 0 {
            1
        } else {
            raw_fingerprint
        };
        let mut idx = (hash ^ (hash >> 32)) as usize & (SIZE - 1);

        loop {
            let slot = &mut self.slots[idx];
            if slot.hash == hash {
                slot.state.update(value);
                return;
            }
            if slot.hash == 0 {
                slot.hash = hash;
                slot.state = StateI::new(value);
                self.names
                    .push(std::str::from_utf8(name).unwrap().to_owned());
                slot.name_id = (self.names.len() - 1) as u16;
                return;
            }
            idx = (idx + 1) & (SIZE - 1);
        }
    }

    #[inline(always)]
    pub(crate) fn find_or_insert(&mut self, name: &[u8], padded_name: &[u8]) -> usize {
        let fingerprint = fingerprint(name, padded_name);

        // Reserve zero as the empty-slot marker without collapsing fingerprints
        // that differ only in their low bit.
        let hash = if fingerprint == 0 { 1 } else { fingerprint };
        let mut idx = (hash ^ (hash >> 32)) as usize & (SIZE - 1);

        loop {
            let slot = &mut self.slots[idx];
            if slot.hash == hash {
                return idx;
            }
            if slot.hash == 0 {
                slot.hash = hash;
                self.names
                    .push(std::str::from_utf8(name).unwrap().to_owned());
                slot.name_id = (self.names.len() - 1) as u16;
                return idx;
            }
            idx = (idx + 1) & (SIZE - 1);
        }
    }

    #[inline(always)]
    pub(crate) fn update_slot(&mut self, slot_idx: usize, value: i16) {
        self.slots[slot_idx].state.update(value);
    }

    pub(crate) fn into_result(mut self) -> Vec<(String, StateF)> {
        let mut result = Vec::with_capacity(self.names.len());
        for slot in self.slots {
            if slot.hash != 0 {
                let name = std::mem::take(&mut self.names[slot.name_id as usize]);
                result.push((name, slot.state.to_f64()));
            }
        }
        result
    }
}

struct SimdFingerprintSlot {
    hash: u64,
    name_id: u16,
}

impl Default for SimdFingerprintSlot {
    fn default() -> Self {
        Self {
            hash: 0,
            name_id: 0,
        }
    }
}

/// Fingerprint lookup with eight independent aggregation stripes per station.
/// A batch always writes one value to each stripe, making SIMD scatter indices
/// unique even when multiple lanes contain the same station.
pub(crate) struct SimdFingerprintTable<const SIZE: usize> {
    slots: Vec<SimdFingerprintSlot>,
    names: Vec<String>,
    mins: Vec<i64>,
    maxs: Vec<i64>,
    counts: Vec<i64>,
    sums: Vec<i64>,
}

impl<const SIZE: usize> SimdFingerprintTable<SIZE> {
    pub(crate) fn new() -> Self {
        assert!(SIZE.is_power_of_two());
        Self {
            slots: (0..SIZE).map(|_| SimdFingerprintSlot::default()).collect(),
            names: Vec::with_capacity(512),
            mins: Vec::with_capacity(512 * 8),
            maxs: Vec::with_capacity(512 * 8),
            counts: Vec::with_capacity(512 * 8),
            sums: Vec::with_capacity(512 * 8),
        }
    }

    #[inline(always)]
    pub(crate) fn find_or_insert(&mut self, name: &[u8], padded_name: &[u8]) -> usize {
        let raw_fingerprint = fingerprint(name, padded_name);
        let hash = if raw_fingerprint == 0 {
            1
        } else {
            raw_fingerprint
        };
        let mut idx = (hash ^ (hash >> 32)) as usize & (SIZE - 1);

        loop {
            let slot = &mut self.slots[idx];
            if slot.hash == hash {
                return slot.name_id as usize;
            }
            if slot.hash == 0 {
                let name_id = self.names.len();
                slot.hash = hash;
                slot.name_id = name_id as u16;
                self.names
                    .push(std::str::from_utf8(name).unwrap().to_owned());
                self.mins.extend([i64::MAX; 8]);
                self.maxs.extend([i64::MIN; 8]);
                self.counts.extend([0; 8]);
                self.sums.extend([0; 8]);
                return name_id;
            }
            idx = (idx + 1) & (SIZE - 1);
        }
    }

    #[inline(always)]
    pub(crate) fn update_batch(&mut self, name_ids: [usize; 8], values: [i16; 8]) {
        let indices =
            Simd::<usize, 8>::from_array(std::array::from_fn(|lane| name_ids[lane] * 8 + lane));
        let values = i64x8::from_array(values.map(i64::from));
        let mins = i64x8::gather_or_default(&self.mins, indices).simd_min(values);
        let maxs = i64x8::gather_or_default(&self.maxs, indices).simd_max(values);
        let counts = i64x8::gather_or_default(&self.counts, indices) + i64x8::splat(1);
        let sums = i64x8::gather_or_default(&self.sums, indices) + values;
        mins.scatter(&mut self.mins, indices);
        maxs.scatter(&mut self.maxs, indices);
        counts.scatter(&mut self.counts, indices);
        sums.scatter(&mut self.sums, indices);
    }

    pub(crate) fn update_partial(&mut self, name_ids: [usize; 8], values: [i16; 8], len: usize) {
        for lane in 0..len {
            let idx = name_ids[lane] * 8 + lane;
            let value = values[lane] as i64;
            self.mins[idx] = self.mins[idx].min(value);
            self.maxs[idx] = self.maxs[idx].max(value);
            self.counts[idx] += 1;
            self.sums[idx] += value;
        }
    }

    pub(crate) fn into_result(self) -> Vec<(String, StateF)> {
        let mut result = Vec::with_capacity(self.names.len());
        for (name_id, name) in self.names.into_iter().enumerate() {
            let start = name_id * 8;
            let mut state = StateI::default();
            for lane in start..start + 8 {
                if self.counts[lane] != 0 {
                    let lane_state = StateI {
                        min: self.mins[lane] as i16,
                        max: self.maxs[lane] as i16,
                        count: self.counts[lane] as u32,
                        sum: self.sums[lane],
                    };
                    state.merge(&lane_state);
                }
            }
            result.push((name, state.to_f64()));
        }
        result
    }
}
