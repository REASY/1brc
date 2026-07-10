use crate::{StateF, StateI};

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

    #[inline]
    pub(crate) fn insert_or_update(
        &mut self,
        name: &[u8],
        padded_name: &[u8],
        value: i16,
    ) {
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
        let fingerprint = first
            ^ last.rotate_left(29)
            ^ (len as u64).wrapping_mul(0x9e3779b97f4a7c15);

        // Reserve zero as the empty-slot marker without collapsing fingerprints
        // that differ only in their low bit.
        let hash = if fingerprint == 0 { 1 } else { fingerprint };
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
                self.names.push(std::str::from_utf8(name).unwrap().to_owned());
                slot.name_id = (self.names.len() - 1) as u16;
                return;
            }
            idx = (idx + 1) & (SIZE - 1);
        }
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
