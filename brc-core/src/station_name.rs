#[allow(unused)]
#[derive(Eq, PartialEq, Debug, Hash)]
struct StationName {
    n1: u64,
    n2: u64,
    n3: u64,
    n4: u64,
    n5: u64,
    n6: u64,
    n7: u64,
    n8: u64,
    n9: u64,
    n10: u64,
    n11: u64,
    n12: u64,
    n13: u64,
}

#[allow(unused)]
impl StationName {
    #[inline]
    fn new(
        n1: u64,
        n2: u64,
        n3: u64,
        n4: u64,
        n5: u64,
        n6: u64,
        n7: u64,
        n8: u64,
        n9: u64,
        n10: u64,
        n11: u64,
        n12: u64,
        n13: u64,
    ) -> StationName {
        StationName {
            n1,
            n2,
            n3,
            n4,
            n5,
            n6,
            n7,
            n8,
            n9,
            n10,
            n11,
            n12,
            n13,
        }
    }

    #[inline]
    fn from(bytes: &[u8]) -> StationName {
        let length = bytes.len();
        match length {
            1 => StationName::new(bytes[0] as u64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
            2 => StationName::new(
                ((bytes[0] as u64) << 8) | (bytes[1] as u64),
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ),
            3 => StationName::new(
                ((bytes[0] as u64) << 16) | ((bytes[1] as u64) << 8) | (bytes[2] as u64),
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ),
            4 => StationName::new(
                ((bytes[0] as u64) << 24)
                    | ((bytes[1] as u64) << 16)
                    | ((bytes[2] as u64) << 8)
                    | (bytes[3] as u64),
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ),
            5 => StationName::new(
                ((bytes[0] as u64) << 32)
                    | ((bytes[1] as u64) << 24)
                    | ((bytes[2] as u64) << 16)
                    | ((bytes[3] as u64) << 8)
                    | (bytes[4] as u64),
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ),
            6 => StationName::new(
                ((bytes[0] as u64) << 40)
                    | ((bytes[1] as u64) << 32)
                    | ((bytes[2] as u64) << 24)
                    | ((bytes[3] as u64) << 16)
                    | ((bytes[4] as u64) << 8)
                    | (bytes[5] as u64),
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ),
            7 => StationName::new(
                ((bytes[0] as u64) << 48)
                    | ((bytes[1] as u64) << 40)
                    | ((bytes[2] as u64) << 32)
                    | ((bytes[3] as u64) << 24)
                    | ((bytes[4] as u64) << 16)
                    | ((bytes[5] as u64) << 8)
                    | (bytes[6] as u64),
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ),
            8 => StationName::new(
                ((bytes[0] as u64) << 56)
                    | ((bytes[1] as u64) << 48)
                    | ((bytes[2] as u64) << 40)
                    | ((bytes[3] as u64) << 32)
                    | ((bytes[4] as u64) << 24)
                    | ((bytes[5] as u64) << 16)
                    | ((bytes[6] as u64) << 8)
                    | (bytes[7] as u64),
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ),
            9 => StationName::new(
                ((bytes[0] as u64) << 56)
                    | ((bytes[1] as u64) << 48)
                    | ((bytes[2] as u64) << 40)
                    | ((bytes[3] as u64) << 32)
                    | ((bytes[4] as u64) << 24)
                    | ((bytes[5] as u64) << 16)
                    | ((bytes[6] as u64) << 8)
                    | (bytes[7] as u64),
                bytes[8] as u64,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ),
            10 => StationName::new(
                ((bytes[0] as u64) << 56)
                    | ((bytes[1] as u64) << 48)
                    | ((bytes[2] as u64) << 40)
                    | ((bytes[3] as u64) << 32)
                    | ((bytes[4] as u64) << 24)
                    | ((bytes[5] as u64) << 16)
                    | ((bytes[6] as u64) << 8)
                    | (bytes[7] as u64),
                ((bytes[8] as u64) << 8) | (bytes[9] as u64),
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ),
            11 => StationName::new(
                ((bytes[0] as u64) << 56)
                    | ((bytes[1] as u64) << 48)
                    | ((bytes[2] as u64) << 40)
                    | ((bytes[3] as u64) << 32)
                    | ((bytes[4] as u64) << 24)
                    | ((bytes[5] as u64) << 16)
                    | ((bytes[6] as u64) << 8)
                    | (bytes[7] as u64),
                ((bytes[8] as u64) << 16) | ((bytes[9] as u64) << 8) | (bytes[10] as u64),
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ),
            12 => StationName::new(
                ((bytes[0] as u64) << 56)
                    | ((bytes[1] as u64) << 48)
                    | ((bytes[2] as u64) << 40)
                    | ((bytes[3] as u64) << 32)
                    | ((bytes[4] as u64) << 24)
                    | ((bytes[5] as u64) << 16)
                    | ((bytes[6] as u64) << 8)
                    | (bytes[7] as u64),
                ((bytes[8] as u64) << 24)
                    | ((bytes[9] as u64) << 16)
                    | ((bytes[10] as u64) << 8)
                    | (bytes[11] as u64),
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ),
            13 => StationName::new(
                ((bytes[0] as u64) << 56)
                    | ((bytes[1] as u64) << 48)
                    | ((bytes[2] as u64) << 40)
                    | ((bytes[3] as u64) << 32)
                    | ((bytes[4] as u64) << 24)
                    | ((bytes[5] as u64) << 16)
                    | ((bytes[6] as u64) << 8)
                    | (bytes[7] as u64),
                ((bytes[8] as u64) << 32)
                    | ((bytes[9] as u64) << 24)
                    | ((bytes[10] as u64) << 16)
                    | ((bytes[11] as u64) << 8)
                    | (bytes[12] as u64),
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ),
            14 => StationName::new(
                ((bytes[0] as u64) << 56)
                    | ((bytes[1] as u64) << 48)
                    | ((bytes[2] as u64) << 40)
                    | ((bytes[3] as u64) << 32)
                    | ((bytes[4] as u64) << 24)
                    | ((bytes[5] as u64) << 16)
                    | ((bytes[6] as u64) << 8)
                    | (bytes[7] as u64),
                ((bytes[8] as u64) << 40)
                    | ((bytes[9] as u64) << 32)
                    | ((bytes[10] as u64) << 24)
                    | ((bytes[11] as u64) << 16)
                    | ((bytes[12] as u64) << 8)
                    | (bytes[13] as u64),
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ),
            15 => StationName::new(
                ((bytes[0] as u64) << 56)
                    | ((bytes[1] as u64) << 48)
                    | ((bytes[2] as u64) << 40)
                    | ((bytes[3] as u64) << 32)
                    | ((bytes[4] as u64) << 24)
                    | ((bytes[5] as u64) << 16)
                    | ((bytes[6] as u64) << 8)
                    | (bytes[7] as u64),
                ((bytes[8] as u64) << 48)
                    | ((bytes[9] as u64) << 40)
                    | ((bytes[10] as u64) << 32)
                    | ((bytes[11] as u64) << 24)
                    | ((bytes[12] as u64) << 16)
                    | ((bytes[13] as u64) << 8)
                    | (bytes[14] as u64),
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ),
            16 => StationName::new(
                ((bytes[0] as u64) << 56)
                    | ((bytes[1] as u64) << 48)
                    | ((bytes[2] as u64) << 40)
                    | ((bytes[3] as u64) << 32)
                    | ((bytes[4] as u64) << 24)
                    | ((bytes[5] as u64) << 16)
                    | ((bytes[6] as u64) << 8)
                    | (bytes[7] as u64),
                ((bytes[8] as u64) << 56)
                    | ((bytes[9] as u64) << 48)
                    | ((bytes[10] as u64) << 40)
                    | ((bytes[11] as u64) << 32)
                    | ((bytes[12] as u64) << 24)
                    | ((bytes[13] as u64) << 16)
                    | ((bytes[14] as u64) << 8)
                    | (bytes[15] as u64),
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ),
            17 => StationName::new(
                ((bytes[0] as u64) << 56)
                    | ((bytes[1] as u64) << 48)
                    | ((bytes[2] as u64) << 40)
                    | ((bytes[3] as u64) << 32)
                    | ((bytes[4] as u64) << 24)
                    | ((bytes[5] as u64) << 16)
                    | ((bytes[6] as u64) << 8)
                    | (bytes[7] as u64),
                ((bytes[8] as u64) << 56)
                    | ((bytes[9] as u64) << 48)
                    | ((bytes[10] as u64) << 40)
                    | ((bytes[11] as u64) << 32)
                    | ((bytes[12] as u64) << 24)
                    | ((bytes[13] as u64) << 16)
                    | ((bytes[14] as u64) << 8)
                    | (bytes[15] as u64),
                bytes[16] as u64,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ),
            18 => StationName::new(
                ((bytes[0] as u64) << 56)
                    | ((bytes[1] as u64) << 48)
                    | ((bytes[2] as u64) << 40)
                    | ((bytes[3] as u64) << 32)
                    | ((bytes[4] as u64) << 24)
                    | ((bytes[5] as u64) << 16)
                    | ((bytes[6] as u64) << 8)
                    | (bytes[7] as u64),
                ((bytes[8] as u64) << 56)
                    | ((bytes[9] as u64) << 48)
                    | ((bytes[10] as u64) << 40)
                    | ((bytes[11] as u64) << 32)
                    | ((bytes[12] as u64) << 24)
                    | ((bytes[13] as u64) << 16)
                    | ((bytes[14] as u64) << 8)
                    | (bytes[15] as u64),
                ((bytes[16] as u64) << 8) | (bytes[17] as u64),
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ),
            19 => StationName::new(
                ((bytes[0] as u64) << 56)
                    | ((bytes[1] as u64) << 48)
                    | ((bytes[2] as u64) << 40)
                    | ((bytes[3] as u64) << 32)
                    | ((bytes[4] as u64) << 24)
                    | ((bytes[5] as u64) << 16)
                    | ((bytes[6] as u64) << 8)
                    | (bytes[7] as u64),
                ((bytes[8] as u64) << 56)
                    | ((bytes[9] as u64) << 48)
                    | ((bytes[10] as u64) << 40)
                    | ((bytes[11] as u64) << 32)
                    | ((bytes[12] as u64) << 24)
                    | ((bytes[13] as u64) << 16)
                    | ((bytes[14] as u64) << 8)
                    | (bytes[15] as u64),
                ((bytes[16] as u64) << 16) | ((bytes[17] as u64) << 8) | (bytes[18] as u64),
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ),
            20 => StationName::new(
                ((bytes[0] as u64) << 56)
                    | ((bytes[1] as u64) << 48)
                    | ((bytes[2] as u64) << 40)
                    | ((bytes[3] as u64) << 32)
                    | ((bytes[4] as u64) << 24)
                    | ((bytes[5] as u64) << 16)
                    | ((bytes[6] as u64) << 8)
                    | (bytes[7] as u64),
                ((bytes[8] as u64) << 56)
                    | ((bytes[9] as u64) << 48)
                    | ((bytes[10] as u64) << 40)
                    | ((bytes[11] as u64) << 32)
                    | ((bytes[12] as u64) << 24)
                    | ((bytes[13] as u64) << 16)
                    | ((bytes[14] as u64) << 8)
                    | (bytes[15] as u64),
                ((bytes[16] as u64) << 24)
                    | ((bytes[17] as u64) << 16)
                    | ((bytes[18] as u64) << 8)
                    | (bytes[19] as u64),
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ),
            21 => StationName::new(
                ((bytes[0] as u64) << 56)
                    | ((bytes[1] as u64) << 48)
                    | ((bytes[2] as u64) << 40)
                    | ((bytes[3] as u64) << 32)
                    | ((bytes[4] as u64) << 24)
                    | ((bytes[5] as u64) << 16)
                    | ((bytes[6] as u64) << 8)
                    | (bytes[7] as u64),
                ((bytes[8] as u64) << 56)
                    | ((bytes[9] as u64) << 48)
                    | ((bytes[10] as u64) << 40)
                    | ((bytes[11] as u64) << 32)
                    | ((bytes[12] as u64) << 24)
                    | ((bytes[13] as u64) << 16)
                    | ((bytes[14] as u64) << 8)
                    | (bytes[15] as u64),
                ((bytes[16] as u64) << 32)
                    | ((bytes[17] as u64) << 24)
                    | ((bytes[18] as u64) << 16)
                    | ((bytes[19] as u64) << 8)
                    | (bytes[20] as u64),
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ),
            22 => StationName::new(
                ((bytes[0] as u64) << 56)
                    | ((bytes[1] as u64) << 48)
                    | ((bytes[2] as u64) << 40)
                    | ((bytes[3] as u64) << 32)
                    | ((bytes[4] as u64) << 24)
                    | ((bytes[5] as u64) << 16)
                    | ((bytes[6] as u64) << 8)
                    | (bytes[7] as u64),
                ((bytes[8] as u64) << 56)
                    | ((bytes[9] as u64) << 48)
                    | ((bytes[10] as u64) << 40)
                    | ((bytes[11] as u64) << 32)
                    | ((bytes[12] as u64) << 24)
                    | ((bytes[13] as u64) << 16)
                    | ((bytes[14] as u64) << 8)
                    | (bytes[15] as u64),
                ((bytes[16] as u64) << 40)
                    | ((bytes[17] as u64) << 32)
                    | ((bytes[18] as u64) << 24)
                    | ((bytes[19] as u64) << 16)
                    | ((bytes[20] as u64) << 8)
                    | (bytes[21] as u64),
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ),
            23 => StationName::new(
                ((bytes[0] as u64) << 56)
                    | ((bytes[1] as u64) << 48)
                    | ((bytes[2] as u64) << 40)
                    | ((bytes[3] as u64) << 32)
                    | ((bytes[4] as u64) << 24)
                    | ((bytes[5] as u64) << 16)
                    | ((bytes[6] as u64) << 8)
                    | (bytes[7] as u64),
                ((bytes[8] as u64) << 56)
                    | ((bytes[9] as u64) << 48)
                    | ((bytes[10] as u64) << 40)
                    | ((bytes[11] as u64) << 32)
                    | ((bytes[12] as u64) << 24)
                    | ((bytes[13] as u64) << 16)
                    | ((bytes[14] as u64) << 8)
                    | (bytes[15] as u64),
                ((bytes[16] as u64) << 48)
                    | ((bytes[17] as u64) << 40)
                    | ((bytes[18] as u64) << 32)
                    | ((bytes[19] as u64) << 24)
                    | ((bytes[20] as u64) << 16)
                    | ((bytes[21] as u64) << 8)
                    | (bytes[22] as u64),
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ),
            24 => StationName::new(
                ((bytes[0] as u64) << 56)
                    | ((bytes[1] as u64) << 48)
                    | ((bytes[2] as u64) << 40)
                    | ((bytes[3] as u64) << 32)
                    | ((bytes[4] as u64) << 24)
                    | ((bytes[5] as u64) << 16)
                    | ((bytes[6] as u64) << 8)
                    | (bytes[7] as u64),
                ((bytes[8] as u64) << 56)
                    | ((bytes[9] as u64) << 48)
                    | ((bytes[10] as u64) << 40)
                    | ((bytes[11] as u64) << 32)
                    | ((bytes[12] as u64) << 24)
                    | ((bytes[13] as u64) << 16)
                    | ((bytes[14] as u64) << 8)
                    | (bytes[15] as u64),
                ((bytes[16] as u64) << 56)
                    | ((bytes[17] as u64) << 48)
                    | ((bytes[18] as u64) << 40)
                    | ((bytes[19] as u64) << 32)
                    | ((bytes[20] as u64) << 24)
                    | ((bytes[21] as u64) << 16)
                    | ((bytes[22] as u64) << 8)
                    | (bytes[23] as u64),
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ),
            25 => StationName::new(
                ((bytes[0] as u64) << 56)
                    | ((bytes[1] as u64) << 48)
                    | ((bytes[2] as u64) << 40)
                    | ((bytes[3] as u64) << 32)
                    | ((bytes[4] as u64) << 24)
                    | ((bytes[5] as u64) << 16)
                    | ((bytes[6] as u64) << 8)
                    | (bytes[7] as u64),
                ((bytes[8] as u64) << 56)
                    | ((bytes[9] as u64) << 48)
                    | ((bytes[10] as u64) << 40)
                    | ((bytes[11] as u64) << 32)
                    | ((bytes[12] as u64) << 24)
                    | ((bytes[13] as u64) << 16)
                    | ((bytes[14] as u64) << 8)
                    | (bytes[15] as u64),
                ((bytes[16] as u64) << 56)
                    | ((bytes[17] as u64) << 48)
                    | ((bytes[18] as u64) << 40)
                    | ((bytes[19] as u64) << 32)
                    | ((bytes[20] as u64) << 24)
                    | ((bytes[21] as u64) << 16)
                    | ((bytes[22] as u64) << 8)
                    | (bytes[23] as u64),
                bytes[24] as u64,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ),
            26 => StationName::new(
                ((bytes[0] as u64) << 56)
                    | ((bytes[1] as u64) << 48)
                    | ((bytes[2] as u64) << 40)
                    | ((bytes[3] as u64) << 32)
                    | ((bytes[4] as u64) << 24)
                    | ((bytes[5] as u64) << 16)
                    | ((bytes[6] as u64) << 8)
                    | (bytes[7] as u64),
                ((bytes[8] as u64) << 56)
                    | ((bytes[9] as u64) << 48)
                    | ((bytes[10] as u64) << 40)
                    | ((bytes[11] as u64) << 32)
                    | ((bytes[12] as u64) << 24)
                    | ((bytes[13] as u64) << 16)
                    | ((bytes[14] as u64) << 8)
                    | (bytes[15] as u64),
                ((bytes[16] as u64) << 56)
                    | ((bytes[17] as u64) << 48)
                    | ((bytes[18] as u64) << 40)
                    | ((bytes[19] as u64) << 32)
                    | ((bytes[20] as u64) << 24)
                    | ((bytes[21] as u64) << 16)
                    | ((bytes[22] as u64) << 8)
                    | (bytes[23] as u64),
                ((bytes[24] as u64) << 8) | (bytes[25] as u64),
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ),
            x => panic!("{}", x),
        }
    }

    #[inline]
    fn from2(bytes: &[u8]) -> StationName {
        let length = bytes.len();
        assert!(length > 1 && length <= 100);

        let n1 = if length > 0 {
            let mut n = 0;
            for i in 0..std::cmp::min(length, 8) {
                n = (n << 8) + bytes[i] as u64;
            }
            n
        } else {
            0
        };

        let n2 = if length > 8 {
            let mut n = 0;
            for i in 8..std::cmp::min(length, 16) {
                n = (n << 8) + bytes[i] as u64;
            }
            n
        } else {
            0
        };

        let n3 = if length > 16 {
            let mut n = 0;
            for i in 16..std::cmp::min(length, 24) {
                n = (n << 8) + bytes[i] as u64;
            }
            n
        } else {
            0
        };

        let n4 = if length > 24 {
            let mut n = 0;
            for i in 24..std::cmp::min(length, 32) {
                n = (n << 8) + bytes[i] as u64;
            }
            n
        } else {
            0
        };

        let n5 = if length > 32 {
            let mut n = 0;
            for i in 32..std::cmp::min(length, 40) {
                n = (n << 8) + bytes[i] as u64;
            }
            n
        } else {
            0
        };

        let n6 = if length > 40 {
            let mut n = 0;
            for i in 40..std::cmp::min(length, 48) {
                n = (n << 8) + bytes[i] as u64;
            }
            n
        } else {
            0
        };

        let n7 = if length > 48 {
            let mut n = 0;
            for i in 48..std::cmp::min(length, 56) {
                n = (n << 8) + bytes[i] as u64;
            }
            n
        } else {
            0
        };

        let n8 = if length > 56 {
            let mut n = 0;
            for i in 56..std::cmp::min(length, 64) {
                n = (n << 8) + bytes[i] as u64;
            }
            n
        } else {
            0
        };

        let n9 = if length > 64 {
            let mut n = 0;
            for i in 64..std::cmp::min(length, 72) {
                n = (n << 8) + bytes[i] as u64;
            }
            n
        } else {
            0
        };

        let n10 = if length > 72 {
            let mut n = 0;
            for i in 72..std::cmp::min(length, 80) {
                n = (n << 8) + bytes[i] as u64;
            }
            n
        } else {
            0
        };

        let n11 = if length > 80 {
            let mut n = 0;
            for i in 80..std::cmp::min(length, 88) {
                n = (n << 8) + bytes[i] as u64;
            }
            n
        } else {
            0
        };

        let n12 = if length > 88 {
            let mut n = 0;
            for i in 88..std::cmp::min(length, 96) {
                n = (n << 8) + bytes[i] as u64;
            }
            n
        } else {
            0
        };

        let n13 = if length > 96 {
            let mut n = 0;
            for i in 96..length {
                n = (n << 8) + bytes[i] as u64;
            }
            n
        } else {
            0
        };

        StationName::new(n1, n2, n3, n4, n5, n6, n7, n8, n9, n10, n11, n12, n13)
    }

    // fn from(bytes: &[u8]) -> StationName {
    //     let length = bytes.len();
    //     assert!(length > 1 && length <= 100);
    //
    //     match length {
    //         0 => panic!(),
    //         1 => {
    //             let n1 = bytes[0] as u64;
    //             StationName::new(n1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0)
    //         },
    //         2 => {
    //             let n1 = (bytes[0] as u64) << 8 + bytes[1] as u64 ;
    //             StationName::new(n1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0)
    //         },
    //         3 => {
    //             let n1 = (bytes[0] as u64) << 16 + (bytes[1] as u64) << 8 +
    // bytes[2] as u64;             StationName::new(n1, 0, 0, 0, 0, 0, 0, 0, 0,
    // 0, 0, 0, 0)         },
    //         4 => {
    //             let n1 = (bytes[0] as u64) << 24 + (bytes[1] as u64) << 16 +
    // (bytes[2] as u64) << 8 + bytes[3] as u64;
    // StationName::new(n1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0)         },
    //         5 => {
    //             let n1 =  (bytes[0] as u64) << 32 + (bytes[1] as u64) << 24 +
    // (bytes[2] as u64) << 16 + (bytes[3] as u64) << 8 + bytes[4] as u64;
    //             StationName::new(n1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0)
    //         },
    //         6 => {
    //             let n1 = (bytes[0] as u64) << 40 + (bytes[1] as u64) << 32 +
    // (bytes[2] as u64) << 24 + (bytes[3] as u64) << 16 + (bytes[4] as u64) << 8 +
    // bytes[5] as u64;             StationName::new(n1, 0, 0, 0, 0, 0, 0, 0, 0,
    // 0, 0, 0, 0)         },
    //         7 => {
    //             let n1 = (bytes[0] as u64) << 48 + (bytes[1] as u64) << 40 +
    // (bytes[2] as u64) << 32 + (bytes[3] as u64) << 24 + (bytes[4] as u64) << 16 +
    // (bytes[5] as u64) << 8 + bytes[6] as u64;
    // StationName::new(n1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0)         },
    //         8 => {
    //             let n1 = (bytes[0] as u64) << 56 + (bytes[1] as u64) << 48 +
    // (bytes[2] as u64) << 40 + (bytes[3] as u64) << 32 + (bytes[4] as u64) << 24 +
    // (bytes[5] as u64) << 16 + (bytes[6] as u64) << 8 + bytes[7] as u64;
    //             StationName::new(n1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0)
    //         },
    //     }
}
