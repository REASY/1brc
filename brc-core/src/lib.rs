mod station_name;
mod table;

use crate::table::Table;
use rustc_hash::FxHashMap;
use std::fmt::Display;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::str::FromStr;

#[derive(Debug)]
pub struct StateF {
    min: f64,
    max: f64,
    count: u32,
    sum: f64,
}

impl Default for StateF {
    fn default() -> Self {
        Self {
            min: f64::MAX,
            max: f64::MIN,
            count: 0,
            sum: 0.0,
        }
    }
}

impl Display for StateF {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let avg = self.sum / (self.count as f64);
        write!(f, "{:.1}/{avg:.1}/{:.1}", self.min, self.max)
    }
}

impl StateF {
    fn update(&mut self, v: f64) {
        self.min = self.min.min(v);
        self.max = self.max.max(v);
        self.count += 1;
        self.sum += v;
    }

    pub fn merge(&mut self, other: &Self) {
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
        self.count += other.count;
        self.sum += other.sum;
    }
}

#[derive(Debug, Clone)]
pub struct StateI {
    min: i16,
    max: i16,
    count: u32,
    sum: i64,
}

impl Default for StateI {
    fn default() -> Self {
        Self {
            min: i16::MAX,
            max: i16::MIN,
            count: 0,
            sum: 0,
        }
    }
}

impl Display for StateI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let avg = self.sum as f64 / 10.0f64 / (self.count as f64);
        write!(f, "{:.1}/{avg:.1}/{:.1}", self.min, self.max)
    }
}

impl StateI {
    fn new(v: i16) -> StateI {
        StateI {
            min: v,
            max: v,
            count: 1,
            sum: v as i64,
        }
    }
    fn update(&mut self, v: i16) {
        self.min = self.min.min(v);
        self.max = self.max.max(v);
        self.count += 1;
        self.sum += v as i64;
    }

    pub fn merge(&mut self, other: &Self) {
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
        self.count += other.count;
        self.sum += other.sum;
    }

    pub fn to_f64(&self) -> StateF {
        StateF {
            min: self.min as f64 / 10.0f64,
            max: self.max as f64 / 10.0f64,
            count: self.count,
            sum: self.sum as f64 / 10.0f64,
        }
    }
}

pub fn sort_result(all: &mut Vec<(String, StateF)>) {
    all.sort_unstable_by(|a, b| a.0.cmp(&b.0));
}

/// Converts a slice of bytes to a string slice.
#[inline]
pub fn byte_to_string(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).unwrap()
}

/// Converts a string in base 10 to a float.
#[inline]
pub fn parse_f64(s: &str) -> f64 {
    f64::from_str(s).unwrap()
}

/// Reads from provided buffered reader line by line, finds station name and temperature and calls processor with found byte slices.
///
/// This is a naive implementation used by [naive_line_by_line_dummy] and [naive_line_by_line]
fn naive_line_by_line0<R: Read + Seek, F>(
    mut rdr: BufReader<R>,
    mut processor: F,
    start: u64,
    end_inclusive: u64,
) where
    F: FnMut(&[u8], &[u8]),
{
    let mut offset: usize = start as usize;
    rdr.seek(SeekFrom::Start(start)).unwrap();

    // Input value ranges are as follows:
    // Station name: non null UTF-8 string of min length 1 character and max length 100 bytes (i.e. this could be 100 one-byte characters, or 50 two-byte characters, etc.)
    // Temperature value: non null double between -99.9 (inclusive) and 99.9 (inclusive), always with one fractional digit
    const MAX_LINE_LENGTH_IN_BYTES: usize = 108; // We actually need 100 + 1 (';') + 5 ("-99.9") + 1 ('\n') = 107

    let mut s: String = String::with_capacity(MAX_LINE_LENGTH_IN_BYTES);
    while offset <= end_inclusive as usize {
        let read_bytes = rdr.read_line(&mut s).expect("Unable to read line");
        // Check whether we reached EOF
        if read_bytes == 0 {
            break;
        }
        offset += read_bytes;
        let slice = s.as_bytes();
        let mut idx: usize = 0;
        // Find station name
        while idx < s.len() && slice[idx] != b';' {
            idx += 1
        }
        let name = &slice[0..idx];
        // The remaining bytes are for temperature
        // We need to subtract 1 from read_bytes because `read_line` includes delimiter as well
        let value = &slice[idx + 1..read_bytes - 1];
        // Call processor to handle the temperature for the station
        processor(name, value);
        // Clear the buffer to make sure next read won't have data from previous read
        s.clear();
    }
}

/// Reads from provided buffered reader station name and temperature and simply accumulates some dummy value.
///
/// This method helps us to understand what is the maximum possible throughput in case of running very simple operation on parsed data.
pub fn naive_line_by_line_dummy<R: Read + Seek>(
    rdr: BufReader<R>,
    start: u64,
    end_inclusive: u64,
    _should_sort: bool,
) -> Vec<(String, StateF)> {
    let mut dummy_result: usize = 0;
    naive_line_by_line0(
        rdr,
        |name: &[u8], t: &[u8]| {
            dummy_result += name.len() + t.len();
        },
        start,
        end_inclusive,
    );

    let mut s = StateF::default();
    s.count = dummy_result as u32;
    vec![("dummy".to_string(), s)]
}

const DEFAULT_HASHMAP_CAPACITY: usize = 10000;

/// Reads from provided buffered reader station name and temperature and aggregates temperature per station.
///
/// The method uses [`byte_to_string`], [`parse_f64`] and [`std::collections::HashMap`] from standard library.
pub fn naive_line_by_line<R: Read + Seek>(
    rdr: BufReader<R>,
    start: u64,
    end_inclusive: u64,
    should_sort: bool,
) -> Vec<(String, StateF)> {
    let mut hs = std::collections::HashMap::with_capacity(DEFAULT_HASHMAP_CAPACITY);
    naive_line_by_line0(
        rdr,
        |name: &[u8], t: &[u8]| {
            // Convert bytes to str
            let station_name: &str = byte_to_string(name);
            let measurement: &str = byte_to_string(t);
            // Parse measurement as f64
            let value = parse_f64(measurement);
            // Insert new state or update existing
            match hs.get_mut(station_name) {
                None => {
                    let mut s = StateF::default();
                    s.update(value);
                    hs.insert(station_name.to_string(), s);
                }
                Some(prev) => prev.update(value),
            }
        },
        start,
        end_inclusive,
    );

    let mut all: Vec<(String, StateF)> = hs.into_iter().collect();
    if should_sort {
        sort_result(&mut all);
    }
    all
}

/// Converts a slice of bytes to a string slice without checking that the string contains valid UTF-8.
#[inline]
pub fn byte_to_string_unsafe(bytes: &[u8]) -> &str {
    unsafe { std::str::from_utf8_unchecked(bytes) }
}

/// Converts byte to a digit
#[inline]
const fn get_digit(b: u8) -> u32 {
    (b as u32).wrapping_sub('0' as u32)
}

/// Converts a float number in the range [-99.9, 99.9] with step 0.1 provided as bytes of str to a scaled i32 value [-999, 999]
///
/// "0.0"   -> 0
/// "-99.9" -> -999
/// "99.9"  -> 999
pub const fn get_as_scaled_integer(bytes: &[u8]) -> i16 {
    let is_negative = bytes[0] == b'-';
    let as_decimal = match (is_negative, bytes.len()) {
        (true, 4) => get_digit(bytes[1]) * 10 + get_digit(bytes[3]),
        (true, 5) => get_digit(bytes[1]) * 100 + get_digit(bytes[2]) * 10 + get_digit(bytes[4]),
        (false, 3) => get_digit(bytes[0]) * 10 + get_digit(bytes[2]),
        (false, 4) => get_digit(bytes[0]) * 100 + get_digit(bytes[1]) * 10 + get_digit(bytes[3]),
        _x => panic!(),
    };
    if is_negative {
        -(as_decimal as i16)
    } else {
        as_decimal as i16
    }
}

/// Reads from provided buffered reader station name and temperature and aggregates temperature per station.
///
/// The method relies on [`naive_line_by_line0`] but uses [`byte_to_string_unsafe`], aggregates data in [`StateI`] and uses [`rustc_hash::FxHashMap`] that makes it ~1.5 times faster than [`naive_line_by_line`]
pub fn naive_line_by_line_v2<R: Read + Seek>(
    rdr: BufReader<R>,
    start: u64,
    end_inclusive: u64,
    should_sort: bool,
) -> Vec<(String, StateF)> {
    let mut hs: FxHashMap<String, StateI> =
        FxHashMap::with_capacity_and_hasher(DEFAULT_HASHMAP_CAPACITY, Default::default());
    naive_line_by_line0(
        rdr,
        |name: &[u8], t: &[u8]| {
            let station_name: &str = byte_to_string_unsafe(name);
            let value = get_as_scaled_integer(t);
            match hs.get_mut(station_name) {
                None => {
                    let mut s = StateI::new(value);
                    s.update(value);
                    hs.insert(station_name.to_string(), s);
                }
                Some(prev) => prev.update(value),
            }
        },
        start,
        end_inclusive,
    );
    let mut all: Vec<(String, StateF)> = hs
        .into_iter()
        .map(|(k, v)| (k.clone(), v.to_f64()))
        .collect();
    if should_sort {
        sort_result(&mut all);
    }
    all
}

#[inline]
fn seek_backward_to_newline<'a, R: Read + Seek>(
    rdr: &mut BufReader<R>,
    buf: &'a [u8],
    read_bytes: usize,
) -> &'a [u8] {
    // Scan backward to find the first new line
    let mut i: usize = 0;
    let mut j: usize = read_bytes - 1;
    while i < read_bytes && buf[j] != b'\n' {
        i += 1;
        j -= 1;
    }

    if i > 0 {
        let pos = i as i64;
        rdr.seek(SeekFrom::Current(-pos))
            .expect("Failed to seek back from current position");
    }

    let valid_buffer = &buf[0..=j];
    valid_buffer
}

const INIT_HASH_VALUE: u64 = 0x517cc1b727220a95;

#[inline]
const fn get_semicolon_pos(w: i64) -> u32 {
    // Check http://www.graphics.stanford.edu/~seander/bithacks.html#ZeroInWord
    let x = w ^ 0x3b3b3b3b3b3b3b3b;
    let t = (x - 0x0101010101010101) & (!x & (0x8080808080808080u64 as i64));
    i64::trailing_zeros(t) >> 3
}

#[inline]
const fn get_semicolon_mask(w: i64) -> i64 {
    // Check http://www.graphics.stanford.edu/~seander/bithacks.html#ZeroInWord
    let x = w ^ 0x3b3b3b3b3b3b3b3b;
    let mask = (x - 0x0101010101010101) & (!x & (0x8080808080808080u64 as i64));
    mask
}

#[inline]
const fn get_decimal_separator_pos(value: i64) -> u32 {
    i64::trailing_zeros(!value & 0x10101000)
}

/// Special method to convert a number in the ascii number into an int without branches created by Quan Anh Mai.
#[inline]
pub const fn to_scaled_integer_branchless(value: i64) -> (i16, i16) {
    let decimal_sep_pos = get_decimal_separator_pos(value) as i32;
    let shift: i32 = 28 - decimal_sep_pos;
    // signed is -1 if negative, 0 otherwise
    let signed = (!value << 59) >> 63;
    let design_mask = !(signed & 0xFF);
    // Align the number to a specific position and transform the ascii to digit value
    let digits = ((value & design_mask) << shift) & 0x0F000F0F00;
    // Now digits is in the form 0xUU00TTHH00 (UU: units digit, TT: tens digit, HH: hundreds digit)
    // 0xUU00TTHH00 * (100 * 0x1000000 + 10 * 0x10000 + 1) =
    // 0x000000UU00TTHH00 + 0x00UU00TTHH000000 * 10 + 0xUU00TTHH00000000 * 100
    let abs_value = ((digits.wrapping_mul(0x640a0001)) >> 32) & 0x3FF;
    let as_int = ((abs_value ^ signed) - signed) as i16;
    let len = ((decimal_sep_pos >> 3) + 3) as i16;
    (as_int, len)
}

const fn get_whole_word(qw: u64, semicolon_pos: usize) -> u64 {
    const MASK: [u64; 9] = [
        0x00,
        0xFF,
        0xFFFF,
        0xFFFFFF,
        0xFFFFFFFF,
        0xFFFFFFFFFF,
        0xFFFFFFFFFFFF,
        0xFFFFFFFFFFFFFF,
        0xFFFFFFFFFFFFFFFF,
    ];
    let mask_value = MASK[semicolon_pos];
    qw & mask_value
}

#[inline]
fn process_buffer_as_bytes<F>(
    processor: &mut F,
    valid_buffer: &[u8],
    mut i: usize,
    n: usize,
    mut next_name_idx: usize,
    should_calculate_hash: bool,
) where
    F: FnMut(&[u8], i16, u64),
{
    while i < n {
        let byte = valid_buffer[i];
        if byte == b';' {
            let mut j: usize = i + 1;
            let start_measurement_idx: usize = j;
            // The shortest temperature as string is "X.Y" that has length = 3
            j += 3;
            // Check remaining 2 bytes that could be because of number like "-XY.Z"
            if valid_buffer[j] != b'\n' {
                j += 1;
            }
            if valid_buffer[j] != b'\n' {
                j += 1;
            }
            let name = &valid_buffer[next_name_idx..i];
            let value = &valid_buffer[start_measurement_idx..j];
            let v = get_as_scaled_integer(value);
            let hash = if should_calculate_hash {
                calculate_hash(&name)
            } else {
                0
            };
            // Call processor to handle the temperature for the station
            processor(name, v, hash);

            // Assign next name index
            if j < n - 1 {
                next_name_idx = j + 1;
            }

            i = j;
        }
        i += 1;
    }
}

#[inline]
fn process_buffer_as_i64<F>(processor: &mut F, valid_buffer: &[u8], should_calculate_hash: bool)
where
    F: FnMut(&[u8], i16, u64),
{
    const BUF_SIZE: usize = std::mem::size_of::<i64>();

    let n = valid_buffer.len();
    let mut i: usize = 0;
    let mut next_name_idx = 0;
    let mut b0: [u8; BUF_SIZE] = [0_u8; BUF_SIZE];
    let mut hash: u64 = 0;
    if should_calculate_hash {
        hash = INIT_HASH_VALUE;
    }
    while i < n - (BUF_SIZE + MAX_MEASUREMENT_LEN) {
        b0.copy_from_slice(&valid_buffer[i..i + BUF_SIZE]);
        let qw0 = i64::from_le_bytes(b0);
        let sp0 = get_semicolon_pos(qw0);
        if sp0 != 8 {
            if should_calculate_hash {
                let word = get_whole_word(qw0 as u64, sp0 as usize);
                hash = hash ^ word;
            }

            let end_exclusive = i + sp0 as usize;
            let name = &valid_buffer[next_name_idx..end_exclusive];

            let start_measurement_idx: usize = end_exclusive + 1;
            b0.copy_from_slice(
                &valid_buffer[start_measurement_idx..start_measurement_idx + BUF_SIZE],
            );
            let qw1 = i64::from_le_bytes(b0);
            let (v, len) = to_scaled_integer_branchless(qw1);

            next_name_idx = start_measurement_idx + len as usize;
            processor(name, v, hash);

            hash = 0;
            if should_calculate_hash {
                hash = INIT_HASH_VALUE;
            }
            i = next_name_idx;
        } else {
            i += 8;
            if should_calculate_hash {
                hash = hash ^ (qw0 as u64);
            }
        }
    }
    // Handle remaining
    process_buffer_as_bytes(
        processor,
        valid_buffer,
        i,
        n,
        next_name_idx,
        should_calculate_hash,
    );
}

#[inline]
fn process_buffer_as_i64_as_java0<F>(processor: &mut F, valid_buffer: &[u8])
where
    F: FnMut(&[u8], i16, u64),
{
    const BUF_SIZE: usize = std::mem::size_of::<i64>();

    let n = valid_buffer.len();
    let mut i: usize = 0;
    let mut next_name_idx = 0;
    let mut b0: [u8; BUF_SIZE] = [0_u8; BUF_SIZE];

    #[inline]
    const fn get_mask(lc: usize) -> u64 {
        const MASK: [u64; 9] = [
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0xFFFFFFFFFFFFFFFF,
        ];
        MASK[lc]
    }
    let mut hash: u64 = INIT_HASH_VALUE;
    while i < n - 3 * BUF_SIZE {
        let qw0 = {
            b0.copy_from_slice(&valid_buffer[i..i + BUF_SIZE]);
            i64::from_le_bytes(b0)
        };
        let m0 = get_semicolon_mask(qw0);

        let qw1 = {
            b0.copy_from_slice(&valid_buffer[i + BUF_SIZE..i + 2 * BUF_SIZE]);
            i64::from_le_bytes(b0)
        };
        let m1 = get_semicolon_mask(qw1);

        // https://github.com/gunnarmorling/1brc/blob/main/src/main/java/dev/morling/onebrc/CalculateAverage_thomaswue.java#L201
        if (m0 | m1) != 0 {
            let letter_count1 = i64::trailing_zeros(m0) >> 3; // value between 1 and 8
            let letter_count2 = i64::trailing_zeros(m1) >> 3; // value between 0 and 8

            hash = hash ^ get_whole_word(qw0 as u64, letter_count1 as usize);

            let len_mask = get_mask(letter_count1 as usize);

            let total_offset = letter_count1 as u64 + (letter_count2 as u64 & len_mask);
            // println!("i: {i}, qw0: {qw0:#08X}, m0: {m0:#08X}, qw1: {qw1:#08X}, m1: {m1:#08X}, total_offset: {total_offset}");

            let end_exclusive = i + total_offset as usize;
            let name = &valid_buffer[next_name_idx..end_exclusive];

            let start_measurement_idx: usize = end_exclusive + 1;
            b0.copy_from_slice(
                &valid_buffer[start_measurement_idx..start_measurement_idx + BUF_SIZE],
            );
            let qw1 = i64::from_le_bytes(b0);
            let (v, len) = to_scaled_integer_branchless(qw1);

            next_name_idx = start_measurement_idx + len as usize;
            processor(name, v, hash);

            i = next_name_idx;

            hash = INIT_HASH_VALUE;
        } else {
            i += 16;
            hash = hash ^ (qw0 as u64);
            hash = hash ^ (qw1 as u64);

            while i < n - BUF_SIZE {
                let qw0 = {
                    b0.copy_from_slice(&valid_buffer[i..i + BUF_SIZE]);
                    i64::from_le_bytes(b0)
                };
                let m0 = get_semicolon_mask(qw0);
                if m0 != 0 {
                    break;
                } else {
                    i += 8;
                }
            }
        }
    }
    // Handle remaining
    process_buffer_as_bytes(processor, valid_buffer, i, n, next_name_idx, false);
}

#[inline]
fn process_buffer_as_i64_unsafe<F>(processor: &mut F, valid_buffer: &[u8])
where
    F: FnMut(&[u8], i16, u64),
{
    let n = valid_buffer.len();
    let mut i: usize = 0;
    let mut next_name_idx = 0;
    let mut ptr: *const u8 = valid_buffer.as_ptr();

    while i < n - 16 {
        let qw0 = unsafe { (ptr as *const i64).read_unaligned() };
        let sp0 = get_semicolon_pos(qw0);
        // println!("i: {i}, qw0: {qw0:#08X}, sp0: {sp0}");
        if sp0 != 8 {
            let end_exclusive = i + sp0 as usize;
            let name = &valid_buffer[next_name_idx..end_exclusive];
            ptr = unsafe { ptr.add((sp0 + 1) as usize) };
            // println!("ptr: {ptr:?}");

            let start_measurement_idx: usize = end_exclusive + 1;
            let qw1 = unsafe { (ptr as *const i64).read_unaligned() };
            // println!("i: {i}, qw0: {qw1:#08X}");
            let (v, len) = to_scaled_integer_branchless(qw1);

            next_name_idx = start_measurement_idx + len as usize;
            processor(name, v, 0);

            ptr = unsafe { ptr.add(len as usize) };

            i = next_name_idx;
        } else {
            ptr = unsafe { ptr.add(8) };
            i += 8;
        }
    }
    // Handle remaining
    process_buffer_as_bytes(processor, valid_buffer, i, n, next_name_idx, false);
}

pub const DEFAULT_BUFFER_SIZE_FOR_LARGE_CHUNK_PARSER: usize = 128 * 1024 * 1024;

/// Reads from provided buffered reader in large chunks, parses it line by line, finds station name and temperature and calls processor with found byte slices.
///
/// This is around 3 times faster than [`naive_line_by_line`] at raw parsing speed.
#[inline]
fn parse_large_chunks_as_bytes0<R: Read + Seek, F>(
    mut rdr: BufReader<R>,
    mut processor: F,
    start: u64,
    end_inclusive: u64,
    buffer_size: usize,
) where
    F: FnMut(&[u8], i16, u64),
{
    let end_incl_usize = end_inclusive as usize;
    let mut offset: usize = start as usize;
    rdr.seek(SeekFrom::Start(start)).unwrap();

    let mut vec: Vec<u8> = vec![0; buffer_size];
    let mut buf = vec.as_mut_slice();
    while offset <= end_incl_usize {
        let mut read_bytes = rdr.read(&mut buf).expect("Unable to read line");
        if read_bytes == 0 {
            break;
        }
        let remaining = end_incl_usize - offset + 1;
        if remaining < buf.len() {
            read_bytes = remaining;
        }
        let valid_buffer = seek_backward_to_newline(&mut rdr, &buf, read_bytes);
        process_buffer_as_bytes(
            &mut processor,
            valid_buffer,
            0,
            valid_buffer.len(),
            0,
            false,
        );
        offset += valid_buffer.len();
    }
}

const MAX_MEASUREMENT_LEN: usize = "-99.9\n".len();

fn parse_large_chunks_as_i64_0<R: Read + Seek, F>(
    mut rdr: BufReader<R>,
    mut processor: F,
    start: u64,
    end_inclusive: u64,
    buffer_size: usize,
    should_calculate_hash: bool,
) where
    F: FnMut(&[u8], i16, u64),
{
    let end_incl_usize = end_inclusive as usize;
    let mut offset: usize = start as usize;
    rdr.seek(SeekFrom::Start(start)).unwrap();

    let mut vec: Vec<u8> = vec![0; buffer_size];
    let mut buf = vec.as_mut_slice();
    while offset <= end_incl_usize {
        let mut read_bytes = rdr.read(&mut buf).expect("Unable to read line");
        if read_bytes == 0 {
            break;
        }
        let remaining = end_incl_usize - offset + 1;
        if remaining < buf.len() {
            read_bytes = remaining;
        }

        let valid_buffer = seek_backward_to_newline(&mut rdr, &buf, read_bytes);
        // println!("Read {read_bytes}, valid_buffer: {}", valid_buffer.len());
        process_buffer_as_i64(&mut processor, valid_buffer, should_calculate_hash);
        offset += valid_buffer.len();
    }
}

fn parse_large_chunks_as_i64_unsafe_0<R: Read + Seek, F>(
    mut rdr: BufReader<R>,
    mut processor: F,
    start: u64,
    end_inclusive: u64,
    buffer_size: usize,
) where
    F: FnMut(&[u8], i16, u64),
{
    let end_incl_usize = end_inclusive as usize;
    let mut offset: usize = start as usize;
    rdr.seek(SeekFrom::Start(start)).unwrap();

    let mut vec: Vec<u8> = vec![0; buffer_size];
    let mut buf = vec.as_mut_slice();
    while offset <= end_incl_usize {
        let mut read_bytes = rdr.read(&mut buf).expect("Unable to read line");
        if read_bytes == 0 {
            break;
        }
        let remaining = end_incl_usize - offset + 1;
        if remaining < buf.len() {
            read_bytes = remaining;
        }
        let valid_buffer = seek_backward_to_newline(&mut rdr, &buf, read_bytes);
        process_buffer_as_i64_unsafe(&mut processor, valid_buffer);
        offset += valid_buffer.len();
    }
}

fn parse_large_chunks_as_i64_as_java0<R: Read + Seek, F>(
    mut rdr: BufReader<R>,
    mut processor: F,
    start: u64,
    end_inclusive: u64,
    buffer_size: usize,
) where
    F: FnMut(&[u8], i16, u64),
{
    let end_incl_usize = end_inclusive as usize;
    let mut offset: usize = start as usize;
    rdr.seek(SeekFrom::Start(start)).unwrap();

    let mut vec: Vec<u8> = vec![0; buffer_size];
    let mut buf = vec.as_mut_slice();
    while offset <= end_incl_usize {
        let mut read_bytes = rdr.read(&mut buf).expect("Unable to read line");
        if read_bytes == 0 {
            break;
        }
        let remaining = end_incl_usize - offset + 1;
        if remaining < buf.len() {
            read_bytes = remaining;
        }
        let valid_buffer = seek_backward_to_newline(&mut rdr, &buf, read_bytes);
        process_buffer_as_i64_as_java0(&mut processor, valid_buffer);
        offset += valid_buffer.len();
    }
}

pub fn parse_large_chunks_as_bytes_dummy<R: Read + Seek>(
    rdr: BufReader<R>,
    start: u64,
    end_inclusive: u64,
    _should_sort: bool,
) -> Vec<(String, StateF)> {
    let mut dummy_result: usize = 0;
    parse_large_chunks_as_bytes0(
        rdr,
        |name: &[u8], t: i16, _| {
            dummy_result += name.len() + t as usize;
        },
        start,
        end_inclusive,
        DEFAULT_BUFFER_SIZE_FOR_LARGE_CHUNK_PARSER,
    );

    let mut s = StateF::default();
    s.count = dummy_result as u32;
    vec![("dummy".to_string(), s)]
}

/// Reads from provided buffered reader station name and temperature and aggregates temperature per station.
///
/// The method relies on [`parse_large_chunks_as_bytes0`] and uses [`byte_to_string_unsafe`] and [`rustc_hash::FxHashMap`] that makes it ~1.8 times faster than [`naive_line_by_line_v2`]
pub fn parse_large_chunks_as_bytes<R: Read + Seek>(
    rdr: BufReader<R>,
    start: u64,
    end_inclusive: u64,
    should_sort: bool,
) -> Vec<(String, StateF)> {
    let mut hs: FxHashMap<Vec<u8>, StateI> =
        FxHashMap::with_capacity_and_hasher(DEFAULT_HASHMAP_CAPACITY, Default::default());
    parse_large_chunks_as_bytes0(
        rdr,
        |name: &[u8], t: i16, _| match hs.get_mut(name) {
            None => {
                hs.insert(name.to_vec(), StateI::new(t));
            }
            Some(prev) => prev.update(t),
        },
        start,
        end_inclusive,
        DEFAULT_BUFFER_SIZE_FOR_LARGE_CHUNK_PARSER,
    );
    let mut all: Vec<(String, StateF)> = hs
        .into_iter()
        .map(|(k, v)| (byte_to_string_unsafe(k.as_slice()).to_string(), v.to_f64()))
        .collect();
    if should_sort {
        sort_result(&mut all);
    }
    all
}

pub fn parse_large_chunks_as_i64_dummy<R: Read + Seek>(
    rdr: BufReader<R>,
    start: u64,
    end_inclusive: u64,
    _should_sort: bool,
) -> Vec<(String, StateF)> {
    let mut dummy_result: usize = 0;
    parse_large_chunks_as_i64_0(
        rdr,
        |name: &[u8], t: i16, _| {
            dummy_result += name.len() + t as usize;
        },
        start,
        end_inclusive,
        DEFAULT_BUFFER_SIZE_FOR_LARGE_CHUNK_PARSER,
        false,
    );

    let mut s = StateF::default();
    s.count = dummy_result as u32;
    vec![("dummy".to_string(), s)]
}

/// Reads from provided buffered reader station name and temperature and aggregates temperature per station.
///
/// The method relies on [`parse_large_chunks_as_i64_0`] and uses [`byte_to_string_unsafe`] and [`rustc_hash::FxHashMap`] that makes it ~1.8 times faster than [`naive_line_by_line_v2`]
pub fn parse_large_chunks_as_i64<R: Read + Seek>(
    rdr: BufReader<R>,
    start: u64,
    end_inclusive: u64,
    should_sort: bool,
) -> Vec<(String, StateF)> {
    let mut hs: FxHashMap<Vec<u8>, StateI> =
        FxHashMap::with_capacity_and_hasher(DEFAULT_HASHMAP_CAPACITY, Default::default());
    parse_large_chunks_as_i64_0(
        rdr,
        |name: &[u8], t: i16, _| match hs.get_mut(name) {
            None => {
                hs.insert(name.to_vec(), StateI::new(t));
            }
            Some(prev) => prev.update(t),
        },
        start,
        end_inclusive,
        DEFAULT_BUFFER_SIZE_FOR_LARGE_CHUNK_PARSER,
        false,
    );
    let mut all: Vec<(String, StateF)> = hs
        .into_iter()
        .map(|(k, v)| (byte_to_string_unsafe(k.as_slice()).to_string(), v.to_f64()))
        .collect();
    if should_sort {
        sort_result(&mut all);
    }
    all
}

pub fn parse_large_chunks_as_i64_v2<R: Read + Seek>(
    rdr: BufReader<R>,
    start: u64,
    end_inclusive: u64,
    should_sort: bool,
) -> Vec<(String, StateF)> {
    const TABLE_SIZE: usize = 10000;

    let mut table: Table<TABLE_SIZE> = Table::new();
    parse_large_chunks_as_i64_0(
        rdr,
        |name: &[u8], t: i16, hash: u64| {
            table.insert_or_update(name, hash, t);
        },
        start,
        end_inclusive,
        DEFAULT_BUFFER_SIZE_FOR_LARGE_CHUNK_PARSER,
        true,
    );
    let mut all: Vec<(String, StateF)> = table.to_result();
    if should_sort {
        sort_result(&mut all);
    }
    all
}

pub fn parse_large_chunks_as_i64_mm(
    valid_buffer: &[u8],
    should_sort: bool,
) -> Vec<(String, StateF)> {
    let mut hs: FxHashMap<String, StateI> =
        FxHashMap::with_capacity_and_hasher(DEFAULT_HASHMAP_CAPACITY, Default::default());

    process_buffer_as_i64_unsafe(
        &mut |name: &[u8], t: i16, _| {
            let station_name: &str = byte_to_string_unsafe(name);
            match hs.get_mut(station_name) {
                None => {
                    hs.insert(station_name.to_string(), StateI::new(t));
                }
                Some(prev) => prev.update(t),
            }
        },
        valid_buffer,
    );

    let mut all: Vec<(String, StateF)> = hs
        .into_iter()
        .map(|(k, v)| (k.clone(), v.to_f64()))
        .collect();
    if should_sort {
        sort_result(&mut all);
    }
    all
}

pub fn parse_large_chunks_as_i64_unsafe<R: Read + Seek>(
    rdr: BufReader<R>,
    start: u64,
    end_inclusive: u64,
    should_sort: bool,
) -> Vec<(String, StateF)> {
    let mut hs: FxHashMap<Vec<u8>, StateI> =
        FxHashMap::with_capacity_and_hasher(DEFAULT_HASHMAP_CAPACITY, Default::default());
    parse_large_chunks_as_i64_unsafe_0(
        rdr,
        |name: &[u8], t: i16, _| match hs.get_mut(name) {
            None => {
                hs.insert(name.to_vec(), StateI::new(t));
            }
            Some(prev) => prev.update(t),
        },
        start,
        end_inclusive,
        DEFAULT_BUFFER_SIZE_FOR_LARGE_CHUNK_PARSER,
    );
    let mut all: Vec<(String, StateF)> = hs
        .into_iter()
        .map(|(k, v)| (byte_to_string_unsafe(k.as_slice()).to_string(), v.to_f64()))
        .collect();
    if should_sort {
        sort_result(&mut all);
    }
    all
}

/// Reads from provided buffered reader in large chunks, parses it line by line using [`memchr::memchr`], finds station name and temperature and calls processor with found byte slices.
///
/// This is around 1.13 times faster than [`parse_large_chunks_as_bytes0`] at raw parsing speed.
fn parse_large_chunks_simd0<R: Read + Seek, F>(
    mut rdr: BufReader<R>,
    mut processor: F,
    start: u64,
    end_inclusive: u64,
    buffer_size: usize,
) where
    F: FnMut(&[u8], &[u8]),
{
    let end_incl_usize = end_inclusive as usize;
    let mut offset: usize = start as usize;
    rdr.seek(SeekFrom::Start(start)).unwrap();

    let mut vec: Vec<u8> = vec![0; buffer_size];
    let mut buf = vec.as_mut_slice();

    while offset <= end_incl_usize {
        let mut read_bytes = rdr.read(&mut buf).expect("Unable to read line");
        if read_bytes == 0 {
            break;
        }
        let remaining = end_incl_usize - offset + 1;
        if remaining < buf.len() {
            read_bytes = remaining;
        }
        let valid_buffer = {
            // Scan backward to find the first new line (0xA)
            let buf_to_scan_backward = &buf[0..read_bytes];
            let idx = memchr::memrchr(b'\n', &buf_to_scan_backward).unwrap();
            let i: usize = buf_to_scan_backward.len() - 1 - idx;
            let j: usize = read_bytes - 1 - i;
            assert!(j < read_bytes, "j: {j}, read_bytes: {read_bytes}");

            if i > 0 {
                let pos = i as i64;
                rdr.seek(SeekFrom::Current(-pos))
                    .expect("Failed to seek back from current position");
            }
            &buf_to_scan_backward[0..=j]
        };
        let mut next_name_idx = 0;
        for it in memchr::memchr_iter(b';', &valid_buffer) {
            let name = &valid_buffer[next_name_idx..it];

            let inner_buf = &valid_buffer[it + 1..];
            let idx = memchr::memchr(b'\n', &inner_buf).unwrap();
            let value = &inner_buf[..idx];
            // Call processor to handle the temperature for the station
            processor(name, value);

            next_name_idx = it + 1 + idx + 1;
        }
        // Tried `memchr::Memchr2` as well, however it is slower, leaving commented code below
        // let mut next_name_idx = 0;
        // let mut next_measurement_idx = 0;
        // let mut station_name_bytes: &[u8] = &[];
        //
        // let mut j: usize = 0;
        // for i in memchr::Memchr2::new(b';', b'\n', valid_buffer) {
        //     if j % 2 == 0 {
        //         station_name_bytes = &valid_buffer[next_name_idx..i];
        //         next_measurement_idx = i + 1;
        //     }
        //     else {
        //         next_name_idx = i + 1;
        //         let measurement_bytes = &valid_buffer[next_measurement_idx..i];
        //         processor(station_name_bytes, measurement_bytes);
        //     }
        //     j += 1;
        // }

        offset += valid_buffer.len();
    }
}

pub fn parse_large_chunks_simd_dummy<R: Read + Seek>(
    rdr: BufReader<R>,
    start: u64,
    end_inclusive: u64,
    _should_sort: bool,
) -> Vec<(String, StateF)> {
    let mut dummy_result: usize = 0;
    parse_large_chunks_simd0(
        rdr,
        |name: &[u8], t: &[u8]| {
            dummy_result += name.len() + t.len();
        },
        start,
        end_inclusive,
        DEFAULT_BUFFER_SIZE_FOR_LARGE_CHUNK_PARSER,
    );

    let mut s = StateF::default();
    s.count = dummy_result as u32;
    vec![("dummy".to_string(), s)]
}

/// Reads from provided buffered reader station name and temperature and aggregates temperature per station.
///
/// The method relies on [`parse_large_chunks_simd0`] and uses [`byte_to_string_unsafe`], [`custom_parse_f64`] and [`rustc_hash::FxHashMap`], could be slightly faster than [`parse_large_chunks_as_bytes`]
pub fn parse_large_chunks_simd<R: Read + Seek>(
    rdr: BufReader<R>,
    start: u64,
    end_inclusive: u64,
    should_sort: bool,
) -> Vec<(String, StateF)> {
    let mut hs: FxHashMap<Vec<u8>, StateI> =
        FxHashMap::with_capacity_and_hasher(DEFAULT_HASHMAP_CAPACITY, Default::default());
    parse_large_chunks_simd0(
        rdr,
        |name: &[u8], t: &[u8]| {
            let value = get_as_scaled_integer(t);
            match hs.get_mut(name) {
                None => {
                    let mut s = StateI::new(value);
                    s.update(value);
                    hs.insert(name.to_vec(), s);
                }
                Some(prev) => prev.update(value),
            }
        },
        start,
        end_inclusive,
        DEFAULT_BUFFER_SIZE_FOR_LARGE_CHUNK_PARSER,
    );
    let mut all: Vec<(String, StateF)> = hs
        .into_iter()
        .map(|(k, v)| (byte_to_string_unsafe(k.as_slice()).to_string(), v.to_f64()))
        .collect();
    if should_sort {
        sort_result(&mut all);
    }
    all
}

/// Holder allows you store slices of bytes inside that can later be used directly as a key in HashMap
///
/// Credits to @R3M4TCH for helping to fix this holder struct
/// https://discord.com/channels/442252698964721669/448238009733742612/1245967276578963498
struct Holder<'a> {
    values: &'a mut [u8],
}

impl<'a> Holder<'a> {
    fn store(&mut self, bytes: &[u8]) -> &'a [u8] {
        let bytes_len = bytes.len();
        let values = std::mem::take(&mut self.values);
        values[..bytes_len].copy_from_slice(bytes);
        // the head will be the piece we wrote to
        let (head, tail) = values.split_at_mut(bytes_len);
        self.values = tail;
        head
    }

    fn new(values: &'static mut [u8]) -> Holder<'a> {
        assert_ne!(0, values.len());

        Holder { values }
    }
}

/// Reads from provided buffered reader station name and temperature and aggregates temperature per station.
///
/// The method relies on [`parse_large_chunks_simd0`] and uses [`StateI`], [`rustc_hash::FxHashMap`], could be slightly faster than [`parse_large_chunks_simd`] or [`parse_large_chunks_as_bytes`]
pub fn parse_large_chunks_simd_v1<R: Read + Seek>(
    rdr: BufReader<R>,
    start: u64,
    end_inclusive: u64,
    should_sort: bool,
) -> Vec<(String, StateF)> {
    let mut hs: FxHashMap<&[u8], StateI> =
        FxHashMap::with_capacity_and_hasher(DEFAULT_HASHMAP_CAPACITY, Default::default());
    let mut holder: Holder = {
        let static_ref: &'static mut [u8] = vec![0; 100 * 10000].leak();
        Holder::new(static_ref)
    };
    parse_large_chunks_simd0(
        rdr,
        |name: &[u8], t: &[u8]| {
            let value = get_as_scaled_integer(t);
            match hs.get_mut(name) {
                None => {
                    let s = StateI::new(value);
                    let name = holder.store(name);
                    hs.insert(name, s);
                }
                Some(prev) => prev.update(value),
            }
        },
        start,
        end_inclusive,
        DEFAULT_BUFFER_SIZE_FOR_LARGE_CHUNK_PARSER,
    );
    let mut all: Vec<(String, StateF)> = hs
        .into_iter()
        .map(|(k, v)| (byte_to_string_unsafe(k).to_string(), v.to_f64()))
        .collect();
    if should_sort {
        sort_result(&mut all);
    }
    all
}

#[inline]
fn calculate_hash(name: &&[u8]) -> u64 {
    const BUF_SIZE: usize = std::mem::size_of::<i64>();
    let mut b0: [u8; BUF_SIZE] = [0_u8; BUF_SIZE];
    let chunks = name.chunks(8);
    let mut hash: u64 = INIT_HASH_VALUE;

    for c in chunks {
        if c.len() == 8 {
            b0.copy_from_slice(c);
        } else {
            let mut i: usize = 0;
            while i < c.len() {
                b0[i] = c[i];
                i += 1;
            }
            while i < 8 {
                b0[i] = 0;
                i += 1;
            }
        }
        let qw0 = i64::from_le_bytes(b0);
        hash = hash ^ (qw0 as u64);
    }
    hash
}

pub fn parse_large_chunks_as_i64_as_java<R: Read + Seek>(
    rdr: BufReader<R>,
    start: u64,
    end_inclusive: u64,
    should_sort: bool,
) -> Vec<(String, StateF)> {
    const TABLE_SIZE: usize = 10000;
    let mut table: Table<TABLE_SIZE> = Table::new();

    parse_large_chunks_as_i64_as_java0(
        rdr,
        |name: &[u8], t: i16, hash: u64| {
            table.insert_or_update(name, hash, t);
        },
        start,
        end_inclusive,
        DEFAULT_BUFFER_SIZE_FOR_LARGE_CHUNK_PARSER,
    );
    let mut all: Vec<(String, StateF)> = table.to_result();
    if should_sort {
        sort_result(&mut all);
    }
    all
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn create_content(stations: &[&str], temperatures: &[&str]) -> String {
        let mut content: String = stations
            .iter()
            .zip(temperatures)
            .map(|(s, t)| format!("{s};{t}"))
            .collect::<Vec<String>>()
            .join("\n");
        // Add ending \n
        content.push_str("\n");
        content
    }

    const STATIONS: [&str; 93] = [
        "A",
        "B",
        "C",
        "hello",
        "Thiès",
        "Yaoundé",
        "Chișinău",
        "Nyugatifelsőszombatfalva",
        "Llanfair­pwllgwyngyll­gogery­chwyrn­drobwll­llan­tysilio­gogo­goch",
        "Taumata­whakatangihanga­koauau­o­tamatea­turi­pukaka­piki­maunga­horo­nuku­pokai­whenua",
        "✨🌟💫🎉🎊🚀🌍🛸🎨📚🎵🎸🎻🎹🎺🎷🧩🛴🚲🏖🏝🏞🏜🌋🏔",
        "Tromsø",
        "Hamilton",
        "Nassau",
        "Bishkek",
        "Dallas",
        "Copenhagen",
        "Ashgabat",
        "Zagreb",
        "Kandi",
        "Chișinău",
        "Sapporo",
        "Da Lat",
        "Malé",
        "Irkutsk",
        "Ürümqi",
        "Los Angeles",
        "Dar es Salaam",
        "Port Vila",
        "Suva",
        "Charlotte",
        "Tirana",
        "Ifrane",
        "Vienna",
        "Port Vila",
        "Bamako",
        "San Antonio",
        "Algiers",
        "Oranjestad",
        "Wau",
        "Nassau",
        "Tirana",
        "Split",
        "Houston",
        "Kankan",
        "Hamilton",
        "Ndola",
        "Ouagadougou",
        "Bosaso",
        "Pontianak",
        "Dublin",
        "Valencia",
        "Ottawa",
        "Djibouti",
        "Bergen",
        "Minsk",
        "Lyon",
        "Phnom Penh",
        "Tallinn",
        "Budapest",
        "Indianapolis",
        "Bouaké",
        "Launceston",
        "Kankan",
        "Panama City",
        "Nicosia",
        "Rabat",
        "Ngaoundéré",
        "Marrakesh",
        "Fairbanks",
        "Prague",
        "Toronto",
        "Palembang",
        "Tabora",
        "Calgary",
        "Tromsø",
        "Dikson",
        "Bujumbura",
        "Alice Springs",
        "Erzurum",
        "Port Moresby",
        "Guatemala City",
        "Philadelphia",
        "Bissau",
        "Hobart",
        "Accra",
        "Abha",
        "Winnipeg",
        "Praia",
        "Palermo",
        "Madrid",
        "Salt Lake City",
        "Denver",
    ];
    const TEMPERATURES: [&str; 93] = [
        "0.1", "0.2", "0.3", "-99.9", "12.3", "0.0", "-12.3", "0.1", "-0.1", "99.9", "12.3", "4.9",
        "14.8", "26.8", "7.8", "29.4", "16.4", "31.7", "15.6", "28.5", "5.1", "18.2", "12.6",
        "28.1", "-7.1", "-6.9", "11.1", "24.4", "27.6", "19.8", "22.7", "20.9", "-0.8", "-9.0",
        "30.3", "11.4", "10.3", "27.8", "16.1", "26.0", "9.1", "7.9", "7.1", "28.4", "46.4", "5.3",
        "31.2", "35.8", "45.3", "23.2", "25.2", "16.2", "-10.6", "47.8", "24.0", "-6.9", "19.2",
        "22.0", "20.2", "13.4", "22.7", "34.3", "8.9", "28.4", "35.6", "5.0", "22.1", "39.9",
        "21.2", "18.8", "-2.9", "11.1", "34.9", "9.8", "4.3", "-6.1", "-14.6", "30.6", "11.6",
        "9.3", "45.2", "23.7", "19.4", "22.4", "21.2", "20.2", "20.4", "-4.3", "18.4", "24.3",
        "9.0", "10.4", "19.9",
    ];

    #[test]
    fn get_digit_works() {
        assert_eq!(0, get_digit(b'0'));
        assert_eq!(1, get_digit(b'1'));
        assert_eq!(2, get_digit(b'2'));
        assert_eq!(3, get_digit(b'3'));
        assert_eq!(4, get_digit(b'4'));
        assert_eq!(5, get_digit(b'5'));
        assert_eq!(6, get_digit(b'6'));
        assert_eq!(7, get_digit(b'7'));
        assert_eq!(8, get_digit(b'8'));
        assert_eq!(9, get_digit(b'9'));
    }

    #[test]
    fn test_naive_line_by_line0() {
        let content = create_content(&STATIONS, &TEMPERATURES);
        let rdr = BufReader::with_capacity(64 * 1024, Cursor::new(content.as_bytes()));
        let mut idx: usize = 0;
        naive_line_by_line0(
            rdr,
            |x, y| {
                assert_eq!(STATIONS[idx].as_bytes(), x);
                assert_eq!(TEMPERATURES[idx].as_bytes(), y);
                idx += 1;
            },
            0,
            (content.len() - 1) as u64,
        );
    }

    #[test]
    fn test_parse_large_chunks_as_bytes0() {
        let content = create_content(&STATIONS, &TEMPERATURES);
        let rdr = BufReader::with_capacity(64 * 1024, Cursor::new(content.as_bytes()));
        let mut idx: usize = 0;
        parse_large_chunks_as_bytes0(
            rdr,
            |x, y, _hash| {
                let expected_v = get_as_scaled_integer(TEMPERATURES[idx].as_bytes());
                let s = STATIONS[idx];
                let str_x = byte_to_string(x);
                println!("Expected: {s}");
                assert_eq!(s.as_bytes(), x, "idx: {idx}, s: {s}, str_x: {str_x}");
                assert_eq!(expected_v, y, "idx: {idx}");
                idx += 1;
            },
            0,
            (content.len() - 1) as u64,
            106,
        );
    }

    #[test]
    fn test_parse_large_chunks_as_i64_0() {
        let content = create_content(&STATIONS, &TEMPERATURES);
        let rdr = BufReader::with_capacity(64 * 1024, Cursor::new(content.as_bytes()));
        let mut idx: usize = 0;
        parse_large_chunks_as_i64_0(
            rdr,
            |x, y, _hash| {
                let expected_v = get_as_scaled_integer(TEMPERATURES[idx].as_bytes());
                let s = STATIONS[idx];
                let str_x = byte_to_string(x);
                println!("Expected: {s}");
                assert_eq!(s.as_bytes(), x, "idx: {idx}, s: {s}, str_x: {str_x}");
                assert_eq!(expected_v, y, "idx: {idx}");
                idx += 1;
            },
            0,
            (content.len() - 1) as u64,
            106,
            false,
        );
    }

    #[test]
    fn test_parse_large_chunks_as_i64_unsafe() {
        let content = create_content(&STATIONS, &TEMPERATURES);
        let rdr = BufReader::with_capacity(64 * 1024, Cursor::new(content.as_bytes()));
        let mut idx: usize = 0;
        parse_large_chunks_as_i64_unsafe_0(
            rdr,
            |x, y, _hash| {
                let expected_v = get_as_scaled_integer(TEMPERATURES[idx].as_bytes());
                let s = STATIONS[idx];
                let str_x = byte_to_string(x);
                println!("Expected: {s}");
                assert_eq!(s.as_bytes(), x, "idx: {idx}, s: {s}, str_x: {str_x}");
                assert_eq!(expected_v, y, "idx: {idx}");
                idx += 1;
            },
            0,
            (content.len() - 1) as u64,
            106,
        );
    }

    #[test]
    fn test_parse_large_chunks_as_i64_as_java0() {
        let content = create_content(&STATIONS, &TEMPERATURES);
        let rdr = BufReader::with_capacity(64 * 1024, Cursor::new(content.as_bytes()));
        let mut idx: usize = 0;
        parse_large_chunks_as_i64_as_java0(
            rdr,
            |x, y, _hash| {
                let expected_v = get_as_scaled_integer(TEMPERATURES[idx].as_bytes());
                let s = STATIONS[idx];
                let str_x = byte_to_string(x);
                println!("Expected: {s}");
                assert_eq!(s.as_bytes(), x, "idx: {idx}, s: {s}, str_x: {str_x}");
                assert_eq!(expected_v, y, "idx: {idx}");
                idx += 1;
            },
            0,
            (content.len() - 1) as u64,
            106,
        );
    }

    #[test]
    fn test_to_scaled_integer_branchless() {
        fn verify(expected_n: i16, next_line: &str) {
            let f = expected_n as f64 / 10 as f64;
            let number_with_newline = format!("{:.1}\n", f);
            let s = format!("{}{}", number_with_newline, next_line);
            let slice = &s.as_str().as_bytes()[0..8];
            let bytes: [u8; 8] = slice.try_into().unwrap();
            let (n, len) = to_scaled_integer_branchless(i64::from_le_bytes(bytes));
            assert_eq!(expected_n, n);
            assert_eq!(number_with_newline.len(), len as usize);
        }
        for i in 0..1000 {
            // When next line is very short
            verify(-i, "A;0.1");
            verify(i, "A;0.1");

            // When next line is not so short
            verify(-i, "hello;-99.9");
            verify(i, "hello;-99.9");
        }
    }

    #[test]
    fn test_parse_large_chunks_simd0() {
        let content = create_content(&STATIONS, &TEMPERATURES);
        let rdr = BufReader::with_capacity(64 * 1024, Cursor::new(content.as_bytes()));
        let mut idx: usize = 0;
        parse_large_chunks_simd0(
            rdr,
            |x, y| {
                assert_eq!(STATIONS[idx].as_bytes(), x);
                assert_eq!(TEMPERATURES[idx].as_bytes(), y);
                idx += 1;
            },
            0,
            (content.len() - 1) as u64,
            106,
        );
    }

    #[test]
    fn test_get_semicolon_pos() {
        assert_eq!(7, get_semicolon_pos(0x3b6f62616c614d31));
        assert_eq!(6, get_semicolon_pos(0x313b6f62616c614d));
        assert_eq!(5, get_semicolon_pos(0x4d313b6f62616c61));
        assert_eq!(4, get_semicolon_pos(0x614d313b6f62616c));
        assert_eq!(3, get_semicolon_pos(0x6c614d313b6f6261));
        assert_eq!(2, get_semicolon_pos(0x616c614d313b6f62));
        assert_eq!(1, get_semicolon_pos(0x62616c614d313b6f));
        assert_eq!(0, get_semicolon_pos(0x6f62616c614d313b));

        // When it is not found, it returns 8
        assert_eq!(8, get_semicolon_pos(0x126f62616c614d31));
        // Case when we have to semicolons, it finds the one that is in smaller part of the number
        assert_eq!(0, get_semicolon_pos(0x413b312e380a413b));
    }
}
