use rustc_hash::FxHashMap;
use std::fmt::Display;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::str::FromStr;

#[derive(Debug)]
pub struct StateF64 {
    min: f64,
    max: f64,
    count: u64,
    sum: f64,
}

impl Default for StateF64 {
    fn default() -> Self {
        Self {
            min: f64::MAX,
            max: f64::MIN,
            count: 0,
            sum: 0.0,
        }
    }
}

impl Display for StateF64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let avg = self.sum / (self.count as f64);
        write!(f, "{:.1}/{avg:.1}/{:.1}", self.min, self.max)
    }
}

impl StateF64 {
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

#[derive(Debug)]
pub struct StateI64 {
    min: i64,
    max: i64,
    count: u64,
    sum: i64,
}

impl Default for StateI64 {
    fn default() -> Self {
        Self {
            min: i64::MAX,
            max: i64::MIN,
            count: 0,
            sum: 0,
        }
    }
}

impl Display for StateI64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let avg = self.sum as f64 / 10.0f64 / (self.count as f64);
        write!(f, "{:.1}/{avg:.1}/{:.1}", self.min, self.max)
    }
}

impl StateI64 {
    fn new(v: i64) -> StateI64 {
        StateI64 {
            min: v,
            max: v,
            count: 1,
            sum: v,
        }
    }
    fn update(&mut self, v: i64) {
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

    pub fn to_f64(&self) -> StateF64 {
        StateF64 {
            min: self.min as f64 / 10.0f64,
            max: self.max as f64 / 10.0f64,
            count: self.count,
            sum: self.sum as f64 / 10.0f64,
        }
    }
}

pub fn sort_result(all: &mut Vec<(String, StateF64)>) {
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
    const MAX_LINE_LENGTH_IN_BYTES: usize = 108; // We actually need 100 + 1 (';') + 5 ("-99.9) = 106

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
        let station_name_bytes = &slice[0..idx];
        // The remaining bytes are for temperature
        // We need to subtract 1 from read_bytes because `read_line` includes delimiter as well
        let measurement_bytes = &slice[idx + 1..read_bytes - 1];
        // Call processor to handle the temperature for the station
        processor(station_name_bytes, measurement_bytes);
        // Clear the buffer to make sure next read won't have data from previous read
        s.clear();
    }
}

/// Reads from provided buffered reader station name and temperature and simply accumulates some dummy value.
///
/// This method helps us to understand what is the maximum possible throughput in case of running very simple operation on found values.
pub fn naive_line_by_line_dummy<R: Read + Seek>(
    rdr: BufReader<R>,
    start: u64,
    end_inclusive: u64,
    _should_sort: bool,
) -> Vec<(String, StateF64)> {
    let mut dummy_result: usize = 0;
    naive_line_by_line0(
        rdr,
        |station_name_bytes, measurement_bytes| {
            dummy_result += station_name_bytes.len() + measurement_bytes.len();
        },
        start,
        end_inclusive,
    );

    let mut s = StateF64::default();
    s.count = dummy_result as u64;
    vec![("dummy".to_string(), s)]
}

const DEFAULT_HASHMAP_CAPACITY: usize = 20000;

/// Reads from provided buffered reader station name and temperature and aggregates temperature per station.
///
/// The method uses [`byte_to_string`], [`parse_f64`] and [`std::collections::HashMap`] from standard library.
pub fn naive_line_by_line<R: Read + Seek>(
    rdr: BufReader<R>,
    start: u64,
    end_inclusive: u64,
    should_sort: bool,
) -> Vec<(String, StateF64)> {
    let mut hs = std::collections::HashMap::with_capacity(DEFAULT_HASHMAP_CAPACITY);
    naive_line_by_line0(
        rdr,
        |station_name_bytes, measurement_bytes| {
            // Convert bytes to str
            let station_name: &str = byte_to_string(station_name_bytes);
            let measurement: &str = byte_to_string(measurement_bytes);
            // Parse measurement as f64
            let value = parse_f64(measurement);
            // Insert new state or update existing
            match hs.get_mut(station_name) {
                None => {
                    let mut s = StateF64::default();
                    s.update(value);
                    hs.insert(station_name.to_string(), s);
                }
                Some(prev) => prev.update(value),
            }
        },
        start,
        end_inclusive,
    );

    let mut all: Vec<(String, StateF64)> = hs.into_iter().collect();
    if should_sort {
        sort_result(&mut all);
    }
    all
}

/// Fixed-size array to be uses in custom float parser for zero and positive f64. The index is a scaled double by a factor of 10
const FLOAT_ZERO_AND_POSITIVE: [f64; 1000] = [
    0.0f64, 0.1f64, 0.2f64, 0.3f64, 0.4f64, 0.5f64, 0.6f64, 0.7f64, 0.8f64, 0.9f64, 1.0f64, 1.1f64,
    1.2f64, 1.3f64, 1.4f64, 1.5f64, 1.6f64, 1.7f64, 1.8f64, 1.9f64, 2.0f64, 2.1f64, 2.2f64, 2.3f64,
    2.4f64, 2.5f64, 2.6f64, 2.7f64, 2.8f64, 2.9f64, 3.0f64, 3.1f64, 3.2f64, 3.3f64, 3.4f64, 3.5f64,
    3.6f64, 3.7f64, 3.8f64, 3.9f64, 4.0f64, 4.1f64, 4.2f64, 4.3f64, 4.4f64, 4.5f64, 4.6f64, 4.7f64,
    4.8f64, 4.9f64, 5.0f64, 5.1f64, 5.2f64, 5.3f64, 5.4f64, 5.5f64, 5.6f64, 5.7f64, 5.8f64, 5.9f64,
    6.0f64, 6.1f64, 6.2f64, 6.3f64, 6.4f64, 6.5f64, 6.6f64, 6.7f64, 6.8f64, 6.9f64, 7.0f64, 7.1f64,
    7.2f64, 7.3f64, 7.4f64, 7.5f64, 7.6f64, 7.7f64, 7.8f64, 7.9f64, 8.0f64, 8.1f64, 8.2f64, 8.3f64,
    8.4f64, 8.5f64, 8.6f64, 8.7f64, 8.8f64, 8.9f64, 9.0f64, 9.1f64, 9.2f64, 9.3f64, 9.4f64, 9.5f64,
    9.6f64, 9.7f64, 9.8f64, 9.9f64, 10.0f64, 10.1f64, 10.2f64, 10.3f64, 10.4f64, 10.5f64, 10.6f64,
    10.7f64, 10.8f64, 10.9f64, 11.0f64, 11.1f64, 11.2f64, 11.3f64, 11.4f64, 11.5f64, 11.6f64,
    11.7f64, 11.8f64, 11.9f64, 12.0f64, 12.1f64, 12.2f64, 12.3f64, 12.4f64, 12.5f64, 12.6f64,
    12.7f64, 12.8f64, 12.9f64, 13.0f64, 13.1f64, 13.2f64, 13.3f64, 13.4f64, 13.5f64, 13.6f64,
    13.7f64, 13.8f64, 13.9f64, 14.0f64, 14.1f64, 14.2f64, 14.3f64, 14.4f64, 14.5f64, 14.6f64,
    14.7f64, 14.8f64, 14.9f64, 15.0f64, 15.1f64, 15.2f64, 15.3f64, 15.4f64, 15.5f64, 15.6f64,
    15.7f64, 15.8f64, 15.9f64, 16.0f64, 16.1f64, 16.2f64, 16.3f64, 16.4f64, 16.5f64, 16.6f64,
    16.7f64, 16.8f64, 16.9f64, 17.0f64, 17.1f64, 17.2f64, 17.3f64, 17.4f64, 17.5f64, 17.6f64,
    17.7f64, 17.8f64, 17.9f64, 18.0f64, 18.1f64, 18.2f64, 18.3f64, 18.4f64, 18.5f64, 18.6f64,
    18.7f64, 18.8f64, 18.9f64, 19.0f64, 19.1f64, 19.2f64, 19.3f64, 19.4f64, 19.5f64, 19.6f64,
    19.7f64, 19.8f64, 19.9f64, 20.0f64, 20.1f64, 20.2f64, 20.3f64, 20.4f64, 20.5f64, 20.6f64,
    20.7f64, 20.8f64, 20.9f64, 21.0f64, 21.1f64, 21.2f64, 21.3f64, 21.4f64, 21.5f64, 21.6f64,
    21.7f64, 21.8f64, 21.9f64, 22.0f64, 22.1f64, 22.2f64, 22.3f64, 22.4f64, 22.5f64, 22.6f64,
    22.7f64, 22.8f64, 22.9f64, 23.0f64, 23.1f64, 23.2f64, 23.3f64, 23.4f64, 23.5f64, 23.6f64,
    23.7f64, 23.8f64, 23.9f64, 24.0f64, 24.1f64, 24.2f64, 24.3f64, 24.4f64, 24.5f64, 24.6f64,
    24.7f64, 24.8f64, 24.9f64, 25.0f64, 25.1f64, 25.2f64, 25.3f64, 25.4f64, 25.5f64, 25.6f64,
    25.7f64, 25.8f64, 25.9f64, 26.0f64, 26.1f64, 26.2f64, 26.3f64, 26.4f64, 26.5f64, 26.6f64,
    26.7f64, 26.8f64, 26.9f64, 27.0f64, 27.1f64, 27.2f64, 27.3f64, 27.4f64, 27.5f64, 27.6f64,
    27.7f64, 27.8f64, 27.9f64, 28.0f64, 28.1f64, 28.2f64, 28.3f64, 28.4f64, 28.5f64, 28.6f64,
    28.7f64, 28.8f64, 28.9f64, 29.0f64, 29.1f64, 29.2f64, 29.3f64, 29.4f64, 29.5f64, 29.6f64,
    29.7f64, 29.8f64, 29.9f64, 30.0f64, 30.1f64, 30.2f64, 30.3f64, 30.4f64, 30.5f64, 30.6f64,
    30.7f64, 30.8f64, 30.9f64, 31.0f64, 31.1f64, 31.2f64, 31.3f64, 31.4f64, 31.5f64, 31.6f64,
    31.7f64, 31.8f64, 31.9f64, 32.0f64, 32.1f64, 32.2f64, 32.3f64, 32.4f64, 32.5f64, 32.6f64,
    32.7f64, 32.8f64, 32.9f64, 33.0f64, 33.1f64, 33.2f64, 33.3f64, 33.4f64, 33.5f64, 33.6f64,
    33.7f64, 33.8f64, 33.9f64, 34.0f64, 34.1f64, 34.2f64, 34.3f64, 34.4f64, 34.5f64, 34.6f64,
    34.7f64, 34.8f64, 34.9f64, 35.0f64, 35.1f64, 35.2f64, 35.3f64, 35.4f64, 35.5f64, 35.6f64,
    35.7f64, 35.8f64, 35.9f64, 36.0f64, 36.1f64, 36.2f64, 36.3f64, 36.4f64, 36.5f64, 36.6f64,
    36.7f64, 36.8f64, 36.9f64, 37.0f64, 37.1f64, 37.2f64, 37.3f64, 37.4f64, 37.5f64, 37.6f64,
    37.7f64, 37.8f64, 37.9f64, 38.0f64, 38.1f64, 38.2f64, 38.3f64, 38.4f64, 38.5f64, 38.6f64,
    38.7f64, 38.8f64, 38.9f64, 39.0f64, 39.1f64, 39.2f64, 39.3f64, 39.4f64, 39.5f64, 39.6f64,
    39.7f64, 39.8f64, 39.9f64, 40.0f64, 40.1f64, 40.2f64, 40.3f64, 40.4f64, 40.5f64, 40.6f64,
    40.7f64, 40.8f64, 40.9f64, 41.0f64, 41.1f64, 41.2f64, 41.3f64, 41.4f64, 41.5f64, 41.6f64,
    41.7f64, 41.8f64, 41.9f64, 42.0f64, 42.1f64, 42.2f64, 42.3f64, 42.4f64, 42.5f64, 42.6f64,
    42.7f64, 42.8f64, 42.9f64, 43.0f64, 43.1f64, 43.2f64, 43.3f64, 43.4f64, 43.5f64, 43.6f64,
    43.7f64, 43.8f64, 43.9f64, 44.0f64, 44.1f64, 44.2f64, 44.3f64, 44.4f64, 44.5f64, 44.6f64,
    44.7f64, 44.8f64, 44.9f64, 45.0f64, 45.1f64, 45.2f64, 45.3f64, 45.4f64, 45.5f64, 45.6f64,
    45.7f64, 45.8f64, 45.9f64, 46.0f64, 46.1f64, 46.2f64, 46.3f64, 46.4f64, 46.5f64, 46.6f64,
    46.7f64, 46.8f64, 46.9f64, 47.0f64, 47.1f64, 47.2f64, 47.3f64, 47.4f64, 47.5f64, 47.6f64,
    47.7f64, 47.8f64, 47.9f64, 48.0f64, 48.1f64, 48.2f64, 48.3f64, 48.4f64, 48.5f64, 48.6f64,
    48.7f64, 48.8f64, 48.9f64, 49.0f64, 49.1f64, 49.2f64, 49.3f64, 49.4f64, 49.5f64, 49.6f64,
    49.7f64, 49.8f64, 49.9f64, 50.0f64, 50.1f64, 50.2f64, 50.3f64, 50.4f64, 50.5f64, 50.6f64,
    50.7f64, 50.8f64, 50.9f64, 51.0f64, 51.1f64, 51.2f64, 51.3f64, 51.4f64, 51.5f64, 51.6f64,
    51.7f64, 51.8f64, 51.9f64, 52.0f64, 52.1f64, 52.2f64, 52.3f64, 52.4f64, 52.5f64, 52.6f64,
    52.7f64, 52.8f64, 52.9f64, 53.0f64, 53.1f64, 53.2f64, 53.3f64, 53.4f64, 53.5f64, 53.6f64,
    53.7f64, 53.8f64, 53.9f64, 54.0f64, 54.1f64, 54.2f64, 54.3f64, 54.4f64, 54.5f64, 54.6f64,
    54.7f64, 54.8f64, 54.9f64, 55.0f64, 55.1f64, 55.2f64, 55.3f64, 55.4f64, 55.5f64, 55.6f64,
    55.7f64, 55.8f64, 55.9f64, 56.0f64, 56.1f64, 56.2f64, 56.3f64, 56.4f64, 56.5f64, 56.6f64,
    56.7f64, 56.8f64, 56.9f64, 57.0f64, 57.1f64, 57.2f64, 57.3f64, 57.4f64, 57.5f64, 57.6f64,
    57.7f64, 57.8f64, 57.9f64, 58.0f64, 58.1f64, 58.2f64, 58.3f64, 58.4f64, 58.5f64, 58.6f64,
    58.7f64, 58.8f64, 58.9f64, 59.0f64, 59.1f64, 59.2f64, 59.3f64, 59.4f64, 59.5f64, 59.6f64,
    59.7f64, 59.8f64, 59.9f64, 60.0f64, 60.1f64, 60.2f64, 60.3f64, 60.4f64, 60.5f64, 60.6f64,
    60.7f64, 60.8f64, 60.9f64, 61.0f64, 61.1f64, 61.2f64, 61.3f64, 61.4f64, 61.5f64, 61.6f64,
    61.7f64, 61.8f64, 61.9f64, 62.0f64, 62.1f64, 62.2f64, 62.3f64, 62.4f64, 62.5f64, 62.6f64,
    62.7f64, 62.8f64, 62.9f64, 63.0f64, 63.1f64, 63.2f64, 63.3f64, 63.4f64, 63.5f64, 63.6f64,
    63.7f64, 63.8f64, 63.9f64, 64.0f64, 64.1f64, 64.2f64, 64.3f64, 64.4f64, 64.5f64, 64.6f64,
    64.7f64, 64.8f64, 64.9f64, 65.0f64, 65.1f64, 65.2f64, 65.3f64, 65.4f64, 65.5f64, 65.6f64,
    65.7f64, 65.8f64, 65.9f64, 66.0f64, 66.1f64, 66.2f64, 66.3f64, 66.4f64, 66.5f64, 66.6f64,
    66.7f64, 66.8f64, 66.9f64, 67.0f64, 67.1f64, 67.2f64, 67.3f64, 67.4f64, 67.5f64, 67.6f64,
    67.7f64, 67.8f64, 67.9f64, 68.0f64, 68.1f64, 68.2f64, 68.3f64, 68.4f64, 68.5f64, 68.6f64,
    68.7f64, 68.8f64, 68.9f64, 69.0f64, 69.1f64, 69.2f64, 69.3f64, 69.4f64, 69.5f64, 69.6f64,
    69.7f64, 69.8f64, 69.9f64, 70.0f64, 70.1f64, 70.2f64, 70.3f64, 70.4f64, 70.5f64, 70.6f64,
    70.7f64, 70.8f64, 70.9f64, 71.0f64, 71.1f64, 71.2f64, 71.3f64, 71.4f64, 71.5f64, 71.6f64,
    71.7f64, 71.8f64, 71.9f64, 72.0f64, 72.1f64, 72.2f64, 72.3f64, 72.4f64, 72.5f64, 72.6f64,
    72.7f64, 72.8f64, 72.9f64, 73.0f64, 73.1f64, 73.2f64, 73.3f64, 73.4f64, 73.5f64, 73.6f64,
    73.7f64, 73.8f64, 73.9f64, 74.0f64, 74.1f64, 74.2f64, 74.3f64, 74.4f64, 74.5f64, 74.6f64,
    74.7f64, 74.8f64, 74.9f64, 75.0f64, 75.1f64, 75.2f64, 75.3f64, 75.4f64, 75.5f64, 75.6f64,
    75.7f64, 75.8f64, 75.9f64, 76.0f64, 76.1f64, 76.2f64, 76.3f64, 76.4f64, 76.5f64, 76.6f64,
    76.7f64, 76.8f64, 76.9f64, 77.0f64, 77.1f64, 77.2f64, 77.3f64, 77.4f64, 77.5f64, 77.6f64,
    77.7f64, 77.8f64, 77.9f64, 78.0f64, 78.1f64, 78.2f64, 78.3f64, 78.4f64, 78.5f64, 78.6f64,
    78.7f64, 78.8f64, 78.9f64, 79.0f64, 79.1f64, 79.2f64, 79.3f64, 79.4f64, 79.5f64, 79.6f64,
    79.7f64, 79.8f64, 79.9f64, 80.0f64, 80.1f64, 80.2f64, 80.3f64, 80.4f64, 80.5f64, 80.6f64,
    80.7f64, 80.8f64, 80.9f64, 81.0f64, 81.1f64, 81.2f64, 81.3f64, 81.4f64, 81.5f64, 81.6f64,
    81.7f64, 81.8f64, 81.9f64, 82.0f64, 82.1f64, 82.2f64, 82.3f64, 82.4f64, 82.5f64, 82.6f64,
    82.7f64, 82.8f64, 82.9f64, 83.0f64, 83.1f64, 83.2f64, 83.3f64, 83.4f64, 83.5f64, 83.6f64,
    83.7f64, 83.8f64, 83.9f64, 84.0f64, 84.1f64, 84.2f64, 84.3f64, 84.4f64, 84.5f64, 84.6f64,
    84.7f64, 84.8f64, 84.9f64, 85.0f64, 85.1f64, 85.2f64, 85.3f64, 85.4f64, 85.5f64, 85.6f64,
    85.7f64, 85.8f64, 85.9f64, 86.0f64, 86.1f64, 86.2f64, 86.3f64, 86.4f64, 86.5f64, 86.6f64,
    86.7f64, 86.8f64, 86.9f64, 87.0f64, 87.1f64, 87.2f64, 87.3f64, 87.4f64, 87.5f64, 87.6f64,
    87.7f64, 87.8f64, 87.9f64, 88.0f64, 88.1f64, 88.2f64, 88.3f64, 88.4f64, 88.5f64, 88.6f64,
    88.7f64, 88.8f64, 88.9f64, 89.0f64, 89.1f64, 89.2f64, 89.3f64, 89.4f64, 89.5f64, 89.6f64,
    89.7f64, 89.8f64, 89.9f64, 90.0f64, 90.1f64, 90.2f64, 90.3f64, 90.4f64, 90.5f64, 90.6f64,
    90.7f64, 90.8f64, 90.9f64, 91.0f64, 91.1f64, 91.2f64, 91.3f64, 91.4f64, 91.5f64, 91.6f64,
    91.7f64, 91.8f64, 91.9f64, 92.0f64, 92.1f64, 92.2f64, 92.3f64, 92.4f64, 92.5f64, 92.6f64,
    92.7f64, 92.8f64, 92.9f64, 93.0f64, 93.1f64, 93.2f64, 93.3f64, 93.4f64, 93.5f64, 93.6f64,
    93.7f64, 93.8f64, 93.9f64, 94.0f64, 94.1f64, 94.2f64, 94.3f64, 94.4f64, 94.5f64, 94.6f64,
    94.7f64, 94.8f64, 94.9f64, 95.0f64, 95.1f64, 95.2f64, 95.3f64, 95.4f64, 95.5f64, 95.6f64,
    95.7f64, 95.8f64, 95.9f64, 96.0f64, 96.1f64, 96.2f64, 96.3f64, 96.4f64, 96.5f64, 96.6f64,
    96.7f64, 96.8f64, 96.9f64, 97.0f64, 97.1f64, 97.2f64, 97.3f64, 97.4f64, 97.5f64, 97.6f64,
    97.7f64, 97.8f64, 97.9f64, 98.0f64, 98.1f64, 98.2f64, 98.3f64, 98.4f64, 98.5f64, 98.6f64,
    98.7f64, 98.8f64, 98.9f64, 99.0f64, 99.1f64, 99.2f64, 99.3f64, 99.4f64, 99.5f64, 99.6f64,
    99.7f64, 99.8f64, 99.9f64,
];

/// Fixed-size array to be uses in custom float parser for zero and negative f64. The index is a scaled double by a factor of 10
const FLOAT_ZERO_AND_NEGATIVE: [f64; 1000] = [
    0.0f64, -0.1f64, -0.2f64, -0.3f64, -0.4f64, -0.5f64, -0.6f64, -0.7f64, -0.8f64, -0.9f64,
    -1.0f64, -1.1f64, -1.2f64, -1.3f64, -1.4f64, -1.5f64, -1.6f64, -1.7f64, -1.8f64, -1.9f64,
    -2.0f64, -2.1f64, -2.2f64, -2.3f64, -2.4f64, -2.5f64, -2.6f64, -2.7f64, -2.8f64, -2.9f64,
    -3.0f64, -3.1f64, -3.2f64, -3.3f64, -3.4f64, -3.5f64, -3.6f64, -3.7f64, -3.8f64, -3.9f64,
    -4.0f64, -4.1f64, -4.2f64, -4.3f64, -4.4f64, -4.5f64, -4.6f64, -4.7f64, -4.8f64, -4.9f64,
    -5.0f64, -5.1f64, -5.2f64, -5.3f64, -5.4f64, -5.5f64, -5.6f64, -5.7f64, -5.8f64, -5.9f64,
    -6.0f64, -6.1f64, -6.2f64, -6.3f64, -6.4f64, -6.5f64, -6.6f64, -6.7f64, -6.8f64, -6.9f64,
    -7.0f64, -7.1f64, -7.2f64, -7.3f64, -7.4f64, -7.5f64, -7.6f64, -7.7f64, -7.8f64, -7.9f64,
    -8.0f64, -8.1f64, -8.2f64, -8.3f64, -8.4f64, -8.5f64, -8.6f64, -8.7f64, -8.8f64, -8.9f64,
    -9.0f64, -9.1f64, -9.2f64, -9.3f64, -9.4f64, -9.5f64, -9.6f64, -9.7f64, -9.8f64, -9.9f64,
    -10.0f64, -10.1f64, -10.2f64, -10.3f64, -10.4f64, -10.5f64, -10.6f64, -10.7f64, -10.8f64,
    -10.9f64, -11.0f64, -11.1f64, -11.2f64, -11.3f64, -11.4f64, -11.5f64, -11.6f64, -11.7f64,
    -11.8f64, -11.9f64, -12.0f64, -12.1f64, -12.2f64, -12.3f64, -12.4f64, -12.5f64, -12.6f64,
    -12.7f64, -12.8f64, -12.9f64, -13.0f64, -13.1f64, -13.2f64, -13.3f64, -13.4f64, -13.5f64,
    -13.6f64, -13.7f64, -13.8f64, -13.9f64, -14.0f64, -14.1f64, -14.2f64, -14.3f64, -14.4f64,
    -14.5f64, -14.6f64, -14.7f64, -14.8f64, -14.9f64, -15.0f64, -15.1f64, -15.2f64, -15.3f64,
    -15.4f64, -15.5f64, -15.6f64, -15.7f64, -15.8f64, -15.9f64, -16.0f64, -16.1f64, -16.2f64,
    -16.3f64, -16.4f64, -16.5f64, -16.6f64, -16.7f64, -16.8f64, -16.9f64, -17.0f64, -17.1f64,
    -17.2f64, -17.3f64, -17.4f64, -17.5f64, -17.6f64, -17.7f64, -17.8f64, -17.9f64, -18.0f64,
    -18.1f64, -18.2f64, -18.3f64, -18.4f64, -18.5f64, -18.6f64, -18.7f64, -18.8f64, -18.9f64,
    -19.0f64, -19.1f64, -19.2f64, -19.3f64, -19.4f64, -19.5f64, -19.6f64, -19.7f64, -19.8f64,
    -19.9f64, -20.0f64, -20.1f64, -20.2f64, -20.3f64, -20.4f64, -20.5f64, -20.6f64, -20.7f64,
    -20.8f64, -20.9f64, -21.0f64, -21.1f64, -21.2f64, -21.3f64, -21.4f64, -21.5f64, -21.6f64,
    -21.7f64, -21.8f64, -21.9f64, -22.0f64, -22.1f64, -22.2f64, -22.3f64, -22.4f64, -22.5f64,
    -22.6f64, -22.7f64, -22.8f64, -22.9f64, -23.0f64, -23.1f64, -23.2f64, -23.3f64, -23.4f64,
    -23.5f64, -23.6f64, -23.7f64, -23.8f64, -23.9f64, -24.0f64, -24.1f64, -24.2f64, -24.3f64,
    -24.4f64, -24.5f64, -24.6f64, -24.7f64, -24.8f64, -24.9f64, -25.0f64, -25.1f64, -25.2f64,
    -25.3f64, -25.4f64, -25.5f64, -25.6f64, -25.7f64, -25.8f64, -25.9f64, -26.0f64, -26.1f64,
    -26.2f64, -26.3f64, -26.4f64, -26.5f64, -26.6f64, -26.7f64, -26.8f64, -26.9f64, -27.0f64,
    -27.1f64, -27.2f64, -27.3f64, -27.4f64, -27.5f64, -27.6f64, -27.7f64, -27.8f64, -27.9f64,
    -28.0f64, -28.1f64, -28.2f64, -28.3f64, -28.4f64, -28.5f64, -28.6f64, -28.7f64, -28.8f64,
    -28.9f64, -29.0f64, -29.1f64, -29.2f64, -29.3f64, -29.4f64, -29.5f64, -29.6f64, -29.7f64,
    -29.8f64, -29.9f64, -30.0f64, -30.1f64, -30.2f64, -30.3f64, -30.4f64, -30.5f64, -30.6f64,
    -30.7f64, -30.8f64, -30.9f64, -31.0f64, -31.1f64, -31.2f64, -31.3f64, -31.4f64, -31.5f64,
    -31.6f64, -31.7f64, -31.8f64, -31.9f64, -32.0f64, -32.1f64, -32.2f64, -32.3f64, -32.4f64,
    -32.5f64, -32.6f64, -32.7f64, -32.8f64, -32.9f64, -33.0f64, -33.1f64, -33.2f64, -33.3f64,
    -33.4f64, -33.5f64, -33.6f64, -33.7f64, -33.8f64, -33.9f64, -34.0f64, -34.1f64, -34.2f64,
    -34.3f64, -34.4f64, -34.5f64, -34.6f64, -34.7f64, -34.8f64, -34.9f64, -35.0f64, -35.1f64,
    -35.2f64, -35.3f64, -35.4f64, -35.5f64, -35.6f64, -35.7f64, -35.8f64, -35.9f64, -36.0f64,
    -36.1f64, -36.2f64, -36.3f64, -36.4f64, -36.5f64, -36.6f64, -36.7f64, -36.8f64, -36.9f64,
    -37.0f64, -37.1f64, -37.2f64, -37.3f64, -37.4f64, -37.5f64, -37.6f64, -37.7f64, -37.8f64,
    -37.9f64, -38.0f64, -38.1f64, -38.2f64, -38.3f64, -38.4f64, -38.5f64, -38.6f64, -38.7f64,
    -38.8f64, -38.9f64, -39.0f64, -39.1f64, -39.2f64, -39.3f64, -39.4f64, -39.5f64, -39.6f64,
    -39.7f64, -39.8f64, -39.9f64, -40.0f64, -40.1f64, -40.2f64, -40.3f64, -40.4f64, -40.5f64,
    -40.6f64, -40.7f64, -40.8f64, -40.9f64, -41.0f64, -41.1f64, -41.2f64, -41.3f64, -41.4f64,
    -41.5f64, -41.6f64, -41.7f64, -41.8f64, -41.9f64, -42.0f64, -42.1f64, -42.2f64, -42.3f64,
    -42.4f64, -42.5f64, -42.6f64, -42.7f64, -42.8f64, -42.9f64, -43.0f64, -43.1f64, -43.2f64,
    -43.3f64, -43.4f64, -43.5f64, -43.6f64, -43.7f64, -43.8f64, -43.9f64, -44.0f64, -44.1f64,
    -44.2f64, -44.3f64, -44.4f64, -44.5f64, -44.6f64, -44.7f64, -44.8f64, -44.9f64, -45.0f64,
    -45.1f64, -45.2f64, -45.3f64, -45.4f64, -45.5f64, -45.6f64, -45.7f64, -45.8f64, -45.9f64,
    -46.0f64, -46.1f64, -46.2f64, -46.3f64, -46.4f64, -46.5f64, -46.6f64, -46.7f64, -46.8f64,
    -46.9f64, -47.0f64, -47.1f64, -47.2f64, -47.3f64, -47.4f64, -47.5f64, -47.6f64, -47.7f64,
    -47.8f64, -47.9f64, -48.0f64, -48.1f64, -48.2f64, -48.3f64, -48.4f64, -48.5f64, -48.6f64,
    -48.7f64, -48.8f64, -48.9f64, -49.0f64, -49.1f64, -49.2f64, -49.3f64, -49.4f64, -49.5f64,
    -49.6f64, -49.7f64, -49.8f64, -49.9f64, -50.0f64, -50.1f64, -50.2f64, -50.3f64, -50.4f64,
    -50.5f64, -50.6f64, -50.7f64, -50.8f64, -50.9f64, -51.0f64, -51.1f64, -51.2f64, -51.3f64,
    -51.4f64, -51.5f64, -51.6f64, -51.7f64, -51.8f64, -51.9f64, -52.0f64, -52.1f64, -52.2f64,
    -52.3f64, -52.4f64, -52.5f64, -52.6f64, -52.7f64, -52.8f64, -52.9f64, -53.0f64, -53.1f64,
    -53.2f64, -53.3f64, -53.4f64, -53.5f64, -53.6f64, -53.7f64, -53.8f64, -53.9f64, -54.0f64,
    -54.1f64, -54.2f64, -54.3f64, -54.4f64, -54.5f64, -54.6f64, -54.7f64, -54.8f64, -54.9f64,
    -55.0f64, -55.1f64, -55.2f64, -55.3f64, -55.4f64, -55.5f64, -55.6f64, -55.7f64, -55.8f64,
    -55.9f64, -56.0f64, -56.1f64, -56.2f64, -56.3f64, -56.4f64, -56.5f64, -56.6f64, -56.7f64,
    -56.8f64, -56.9f64, -57.0f64, -57.1f64, -57.2f64, -57.3f64, -57.4f64, -57.5f64, -57.6f64,
    -57.7f64, -57.8f64, -57.9f64, -58.0f64, -58.1f64, -58.2f64, -58.3f64, -58.4f64, -58.5f64,
    -58.6f64, -58.7f64, -58.8f64, -58.9f64, -59.0f64, -59.1f64, -59.2f64, -59.3f64, -59.4f64,
    -59.5f64, -59.6f64, -59.7f64, -59.8f64, -59.9f64, -60.0f64, -60.1f64, -60.2f64, -60.3f64,
    -60.4f64, -60.5f64, -60.6f64, -60.7f64, -60.8f64, -60.9f64, -61.0f64, -61.1f64, -61.2f64,
    -61.3f64, -61.4f64, -61.5f64, -61.6f64, -61.7f64, -61.8f64, -61.9f64, -62.0f64, -62.1f64,
    -62.2f64, -62.3f64, -62.4f64, -62.5f64, -62.6f64, -62.7f64, -62.8f64, -62.9f64, -63.0f64,
    -63.1f64, -63.2f64, -63.3f64, -63.4f64, -63.5f64, -63.6f64, -63.7f64, -63.8f64, -63.9f64,
    -64.0f64, -64.1f64, -64.2f64, -64.3f64, -64.4f64, -64.5f64, -64.6f64, -64.7f64, -64.8f64,
    -64.9f64, -65.0f64, -65.1f64, -65.2f64, -65.3f64, -65.4f64, -65.5f64, -65.6f64, -65.7f64,
    -65.8f64, -65.9f64, -66.0f64, -66.1f64, -66.2f64, -66.3f64, -66.4f64, -66.5f64, -66.6f64,
    -66.7f64, -66.8f64, -66.9f64, -67.0f64, -67.1f64, -67.2f64, -67.3f64, -67.4f64, -67.5f64,
    -67.6f64, -67.7f64, -67.8f64, -67.9f64, -68.0f64, -68.1f64, -68.2f64, -68.3f64, -68.4f64,
    -68.5f64, -68.6f64, -68.7f64, -68.8f64, -68.9f64, -69.0f64, -69.1f64, -69.2f64, -69.3f64,
    -69.4f64, -69.5f64, -69.6f64, -69.7f64, -69.8f64, -69.9f64, -70.0f64, -70.1f64, -70.2f64,
    -70.3f64, -70.4f64, -70.5f64, -70.6f64, -70.7f64, -70.8f64, -70.9f64, -71.0f64, -71.1f64,
    -71.2f64, -71.3f64, -71.4f64, -71.5f64, -71.6f64, -71.7f64, -71.8f64, -71.9f64, -72.0f64,
    -72.1f64, -72.2f64, -72.3f64, -72.4f64, -72.5f64, -72.6f64, -72.7f64, -72.8f64, -72.9f64,
    -73.0f64, -73.1f64, -73.2f64, -73.3f64, -73.4f64, -73.5f64, -73.6f64, -73.7f64, -73.8f64,
    -73.9f64, -74.0f64, -74.1f64, -74.2f64, -74.3f64, -74.4f64, -74.5f64, -74.6f64, -74.7f64,
    -74.8f64, -74.9f64, -75.0f64, -75.1f64, -75.2f64, -75.3f64, -75.4f64, -75.5f64, -75.6f64,
    -75.7f64, -75.8f64, -75.9f64, -76.0f64, -76.1f64, -76.2f64, -76.3f64, -76.4f64, -76.5f64,
    -76.6f64, -76.7f64, -76.8f64, -76.9f64, -77.0f64, -77.1f64, -77.2f64, -77.3f64, -77.4f64,
    -77.5f64, -77.6f64, -77.7f64, -77.8f64, -77.9f64, -78.0f64, -78.1f64, -78.2f64, -78.3f64,
    -78.4f64, -78.5f64, -78.6f64, -78.7f64, -78.8f64, -78.9f64, -79.0f64, -79.1f64, -79.2f64,
    -79.3f64, -79.4f64, -79.5f64, -79.6f64, -79.7f64, -79.8f64, -79.9f64, -80.0f64, -80.1f64,
    -80.2f64, -80.3f64, -80.4f64, -80.5f64, -80.6f64, -80.7f64, -80.8f64, -80.9f64, -81.0f64,
    -81.1f64, -81.2f64, -81.3f64, -81.4f64, -81.5f64, -81.6f64, -81.7f64, -81.8f64, -81.9f64,
    -82.0f64, -82.1f64, -82.2f64, -82.3f64, -82.4f64, -82.5f64, -82.6f64, -82.7f64, -82.8f64,
    -82.9f64, -83.0f64, -83.1f64, -83.2f64, -83.3f64, -83.4f64, -83.5f64, -83.6f64, -83.7f64,
    -83.8f64, -83.9f64, -84.0f64, -84.1f64, -84.2f64, -84.3f64, -84.4f64, -84.5f64, -84.6f64,
    -84.7f64, -84.8f64, -84.9f64, -85.0f64, -85.1f64, -85.2f64, -85.3f64, -85.4f64, -85.5f64,
    -85.6f64, -85.7f64, -85.8f64, -85.9f64, -86.0f64, -86.1f64, -86.2f64, -86.3f64, -86.4f64,
    -86.5f64, -86.6f64, -86.7f64, -86.8f64, -86.9f64, -87.0f64, -87.1f64, -87.2f64, -87.3f64,
    -87.4f64, -87.5f64, -87.6f64, -87.7f64, -87.8f64, -87.9f64, -88.0f64, -88.1f64, -88.2f64,
    -88.3f64, -88.4f64, -88.5f64, -88.6f64, -88.7f64, -88.8f64, -88.9f64, -89.0f64, -89.1f64,
    -89.2f64, -89.3f64, -89.4f64, -89.5f64, -89.6f64, -89.7f64, -89.8f64, -89.9f64, -90.0f64,
    -90.1f64, -90.2f64, -90.3f64, -90.4f64, -90.5f64, -90.6f64, -90.7f64, -90.8f64, -90.9f64,
    -91.0f64, -91.1f64, -91.2f64, -91.3f64, -91.4f64, -91.5f64, -91.6f64, -91.7f64, -91.8f64,
    -91.9f64, -92.0f64, -92.1f64, -92.2f64, -92.3f64, -92.4f64, -92.5f64, -92.6f64, -92.7f64,
    -92.8f64, -92.9f64, -93.0f64, -93.1f64, -93.2f64, -93.3f64, -93.4f64, -93.5f64, -93.6f64,
    -93.7f64, -93.8f64, -93.9f64, -94.0f64, -94.1f64, -94.2f64, -94.3f64, -94.4f64, -94.5f64,
    -94.6f64, -94.7f64, -94.8f64, -94.9f64, -95.0f64, -95.1f64, -95.2f64, -95.3f64, -95.4f64,
    -95.5f64, -95.6f64, -95.7f64, -95.8f64, -95.9f64, -96.0f64, -96.1f64, -96.2f64, -96.3f64,
    -96.4f64, -96.5f64, -96.6f64, -96.7f64, -96.8f64, -96.9f64, -97.0f64, -97.1f64, -97.2f64,
    -97.3f64, -97.4f64, -97.5f64, -97.6f64, -97.7f64, -97.8f64, -97.9f64, -98.0f64, -98.1f64,
    -98.2f64, -98.3f64, -98.4f64, -98.5f64, -98.6f64, -98.7f64, -98.8f64, -98.9f64, -99.0f64,
    -99.1f64, -99.2f64, -99.3f64, -99.4f64, -99.5f64, -99.6f64, -99.7f64, -99.8f64, -99.9f64,
];

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
#[inline]
pub fn get_as_scaled_integer(bytes: &[u8]) -> i32 {
    let is_negative = bytes[0] == b'-';
    let as_decimal = match (is_negative, bytes.len()) {
        (true, 4) => get_digit(bytes[1]) * 10 + get_digit(bytes[3]),
        (true, 5) => get_digit(bytes[1]) * 100 + get_digit(bytes[2]) * 10 + get_digit(bytes[4]),
        (false, 3) => get_digit(bytes[0]) * 10 + get_digit(bytes[2]),
        (false, 4) => get_digit(bytes[0]) * 100 + get_digit(bytes[1]) * 10 + get_digit(bytes[3]),
        x => panic!("x: {:?}", x),
    };
    if is_negative {
        -(as_decimal as i32)
    } else {
        as_decimal as i32
    }
}

/// Custom parser of a string slice to f64. This is faster than built-in one for our constrained range.
#[inline]
pub fn custom_parse_f64(s: &str) -> f64 {
    let scaled_by_10_as_idx = get_as_scaled_integer(s.as_bytes());
    if scaled_by_10_as_idx >= 0 {
        FLOAT_ZERO_AND_POSITIVE[scaled_by_10_as_idx as usize]
    } else {
        FLOAT_ZERO_AND_NEGATIVE[(-scaled_by_10_as_idx) as usize]
    }
}

#[inline]
pub fn custom_parse_f64_simd(s: &str) -> f64 {
    let bytes = s.as_bytes();
    let is_negative = bytes[0] == b'-';
    let as_decimal = unsafe {
        match (is_negative, bytes.len()) {
            (true, 4) => simd_calculate_decimal(&[bytes[1], b'0', bytes[3], b'0']),
            (true, 5) => simd_calculate_decimal(&[bytes[1], bytes[2], bytes[4], b'0']),
            (false, 3) => simd_calculate_decimal(&[bytes[0], b'0', bytes[2], b'0']),
            (false, 4) => simd_calculate_decimal(&[bytes[0], bytes[1], bytes[3], b'0']),
            x => panic!("Unexpected input format: {:?}", x),
        }
    };
    if is_negative {
        FLOAT_ZERO_AND_NEGATIVE[as_decimal as usize]
    } else {
        FLOAT_ZERO_AND_POSITIVE[as_decimal as usize]
    }
}

#[inline]
unsafe fn simd_calculate_decimal(bytes: &[u8; 4]) -> i32 {
    use std::arch::x86_64::*;

    let digit_offsets = _mm_set1_epi8(b'0' as i8);
    let byte_vec = _mm_loadu_si128(bytes.as_ptr() as *const __m128i);
    let digits = _mm_sub_epi8(byte_vec, digit_offsets);

    let multiplier_vec = _mm_setr_epi16(100, 10, 1, 0, 0, 0, 0, 0);
    let digits_16 = _mm_cvtepu8_epi16(digits);

    let multiplied = _mm_madd_epi16(digits_16, multiplier_vec);

    let summed = _mm_hadd_epi16(multiplied, _mm_setzero_si128());
    let result = _mm_extract_epi16::<0>(summed) + _mm_extract_epi16::<1>(summed);

    result as i32
}

/// Reads from provided buffered reader station name and temperature and aggregates temperature per station.
///
/// The method relies on [`naive_line_by_line0`] but uses [`byte_to_string_unsafe`], [`custom_parse_f64`] and [`rustc_hash::FxHashMap`] that makes it ~1.5 times faster than [`naive_line_by_line`]
pub fn naive_line_by_line_v2<R: Read + Seek>(
    rdr: BufReader<R>,
    start: u64,
    end_inclusive: u64,
    should_sort: bool,
) -> Vec<(String, StateF64)> {
    let mut hs: FxHashMap<String, StateF64> =
        FxHashMap::with_capacity_and_hasher(DEFAULT_HASHMAP_CAPACITY, Default::default());
    naive_line_by_line0(
        rdr,
        |station_name_bytes, measurement_bytes| {
            let station_name: &str = byte_to_string_unsafe(station_name_bytes);
            let measurement: &str = byte_to_string_unsafe(measurement_bytes);
            let value = custom_parse_f64(measurement);
            match hs.get_mut(station_name) {
                None => {
                    let mut s = StateF64::default();
                    s.update(value);
                    hs.insert(station_name.to_string(), s);
                }
                Some(prev) => prev.update(value),
            }
        },
        start,
        end_inclusive,
    );
    let mut all: Vec<(String, StateF64)> = hs.into_iter().collect();
    if should_sort {
        sort_result(&mut all);
    }
    all
}

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

const DEFAULT_BUFFER_SIZE_FOR_LARGE_CHUNK_PARSER: usize = 64 * 1024 * 1024;

/// Reads from provided buffered reader in large chunks, parses it line by line, finds station name and temperature and calls processor with found byte slices.
///
/// This is around 3 times faster than [`naive_line_by_line`] at raw parsing speed.
fn parse_large_chunks0<R: Read + Seek, F>(
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

        let valid_buffer = seek_backward_to_newline(&mut rdr, &buf, read_bytes);
        let n = valid_buffer.len();

        let mut i: usize = 0;
        let mut next_name_idx = 0;
        while i < n {
            if valid_buffer[i] == b';' {
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
                let station_name_bytes = &valid_buffer[next_name_idx..i];
                let measurement_bytes = &valid_buffer[start_measurement_idx..j];
                // Call processor to handle the temperature for the station
                processor(station_name_bytes, measurement_bytes);

                // Assign next name index
                if j < n - 1 {
                    next_name_idx = j + 1;
                }

                i = j;
            }
            i += 1;
        }

        offset += n;
    }
}

pub fn parse_large_chunks_dummy<R: Read + Seek>(
    rdr: BufReader<R>,
    start: u64,
    end_inclusive: u64,
    _should_sort: bool,
) -> Vec<(String, StateF64)> {
    let mut dummy_result: usize = 0;
    parse_large_chunks0(
        rdr,
        |station_name_bytes, measurement_bytes| {
            dummy_result += station_name_bytes.len() + measurement_bytes.len();
        },
        start,
        end_inclusive,
        DEFAULT_BUFFER_SIZE_FOR_LARGE_CHUNK_PARSER,
    );

    let mut s = StateF64::default();
    s.count = dummy_result as u64;
    vec![("dummy".to_string(), s)]
}

/// Reads from provided buffered reader station name and temperature and aggregates temperature per station.
///
/// The method relies on [`parse_large_chunks0`] and uses [`byte_to_string_unsafe`], [`custom_parse_f64`] and [`rustc_hash::FxHashMap`] that makes it ~1.8 times faster than [`naive_line_by_line_v2`]
pub fn parse_large_chunks<R: Read + Seek>(
    rdr: BufReader<R>,
    start: u64,
    end_inclusive: u64,
    should_sort: bool,
) -> Vec<(String, StateF64)> {
    let mut hs: FxHashMap<String, StateF64> =
        FxHashMap::with_capacity_and_hasher(DEFAULT_HASHMAP_CAPACITY, Default::default());
    parse_large_chunks0(
        rdr,
        |station_name_bytes, measurement_bytes| {
            let station_name: &str = byte_to_string_unsafe(station_name_bytes);
            let measurement: &str = byte_to_string_unsafe(measurement_bytes);
            let value = custom_parse_f64(measurement);
            match hs.get_mut(station_name) {
                None => {
                    let mut s = StateF64::default();
                    s.update(value);
                    hs.insert(station_name.to_string(), s);
                }
                Some(prev) => prev.update(value),
            }
        },
        start,
        end_inclusive,
        DEFAULT_BUFFER_SIZE_FOR_LARGE_CHUNK_PARSER,
    );
    let mut all: Vec<(String, StateF64)> = hs.into_iter().collect();
    if should_sort {
        sort_result(&mut all);
    }
    all
}

/// Reads from provided buffered reader in large chunks, parses it line by line using [`memchr::memchr`], finds station name and temperature and calls processor with found byte slices.
///
/// This is around 1.13 times faster than [`parse_large_chunks0`] at raw parsing speed.
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
            let station_name_bytes = &valid_buffer[next_name_idx..it];

            let inner_buf = &valid_buffer[it + 1..];
            let idx = memchr::memchr(b'\n', &inner_buf).unwrap();
            let measurement_bytes = &inner_buf[..idx];
            // Call processor to handle the temperature for the station
            processor(station_name_bytes, measurement_bytes);

            next_name_idx = it + 1 + idx + 1;
        }
        offset += valid_buffer.len();
    }
}

pub fn parse_large_chunks_simd_dummy<R: Read + Seek>(
    rdr: BufReader<R>,
    start: u64,
    end_inclusive: u64,
    _should_sort: bool,
) -> Vec<(String, StateF64)> {
    let mut dummy_result: usize = 0;
    parse_large_chunks_simd0(
        rdr,
        |station_name_bytes, measurement_bytes| {
            dummy_result += station_name_bytes.len() + measurement_bytes.len();
        },
        start,
        end_inclusive,
        64 * 1024 * 1024,
    );

    let mut s = StateF64::default();
    s.count = dummy_result as u64;
    vec![("dummy".to_string(), s)]
}

/// Reads from provided buffered reader station name and temperature and aggregates temperature per station.
///
/// The method relies on [`parse_large_chunks_simd0`] and uses [`byte_to_string_unsafe`], [`custom_parse_f64`] and [`rustc_hash::FxHashMap`], could be slightly faster than [`parse_large_chunks`]
pub fn parse_large_chunks_simd<R: Read + Seek>(
    rdr: BufReader<R>,
    start: u64,
    end_inclusive: u64,
    should_sort: bool,
) -> Vec<(String, StateF64)> {
    let mut hs: FxHashMap<String, StateF64> =
        FxHashMap::with_capacity_and_hasher(DEFAULT_HASHMAP_CAPACITY, Default::default());
    parse_large_chunks_simd0(
        rdr,
        |station_name_bytes, measurement_bytes| {
            let station_name: &str = byte_to_string_unsafe(station_name_bytes);
            let measurement: &str = byte_to_string_unsafe(measurement_bytes);
            let value = custom_parse_f64(measurement);
            match hs.get_mut(station_name) {
                None => {
                    let mut s = StateF64::default();
                    s.update(value);
                    hs.insert(station_name.to_string(), s);
                }
                Some(prev) => prev.update(value),
            }
        },
        start,
        end_inclusive,
        DEFAULT_BUFFER_SIZE_FOR_LARGE_CHUNK_PARSER,
    );
    let mut all: Vec<(String, StateF64)> = hs.into_iter().collect();
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
/// The method relies on [`parse_large_chunks_simd0`] and uses [`StateI64`], [`hashbrown::HashMap`], could be slightly faster than [`parse_large_chunks_simd`] or [`parse_large_chunks`]
pub fn parse_large_chunks_v1<R: Read + Seek>(
    rdr: BufReader<R>,
    start: u64,
    end_inclusive: u64,
    should_sort: bool,
) -> Vec<(String, StateF64)> {
    let mut hs: hashbrown::HashMap<&[u8], StateI64> =
        hashbrown::HashMap::with_capacity(DEFAULT_HASHMAP_CAPACITY);
    let mut holder: Holder = {
        let static_ref: &'static mut [u8] = vec![0; 100 * 10000].leak();
        Holder::new(static_ref)
    };

    parse_large_chunks_simd0(
        rdr,
        |station_name_bytes, measurement_bytes| {
            let value = get_as_scaled_integer(measurement_bytes);
            match hs.get_mut(station_name_bytes) {
                None => {
                    let s = StateI64::new(value as i64);
                    let name = holder.store(station_name_bytes);
                    hs.insert(name, s);
                }
                Some(prev) => prev.update(value as i64),
            }
        },
        start,
        end_inclusive,
        DEFAULT_BUFFER_SIZE_FOR_LARGE_CHUNK_PARSER,
    );
    let mut all: Vec<(String, StateF64)> = hs
        .into_iter()
        .map(|(k, v)| (byte_to_string_unsafe(k).to_string(), v.to_f64()))
        .collect();
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

    const STATIONS: [&str; 90] = [
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
    const TEMPERATURES: [&str; 90] = [
        "-99.9", "12.3", "0.0", "-12.3", "0.1", "-0.1", "99.9", "12.3", "4.9", "14.8", "26.8",
        "7.8", "29.4", "16.4", "31.7", "15.6", "28.5", "5.1", "18.2", "12.6", "28.1", "-7.1",
        "-6.9", "11.1", "24.4", "27.6", "19.8", "22.7", "20.9", "-0.8", "-9.0", "30.3", "11.4",
        "10.3", "27.8", "16.1", "26.0", "9.1", "7.9", "7.1", "28.4", "46.4", "5.3", "31.2", "35.8",
        "45.3", "23.2", "25.2", "16.2", "-10.6", "47.8", "24.0", "-6.9", "19.2", "22.0", "20.2",
        "13.4", "22.7", "34.3", "8.9", "28.4", "35.6", "5.0", "22.1", "39.9", "21.2", "18.8",
        "-2.9", "11.1", "34.9", "9.8", "4.3", "-6.1", "-14.6", "30.6", "11.6", "9.3", "45.2",
        "23.7", "19.4", "22.4", "21.2", "20.2", "20.4", "-4.3", "18.4", "24.3", "9.0", "10.4",
        "19.9",
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
    fn custom_parse_f64_works() {
        // Verify positive and negative numbers in the range [-99.9, 99.9] with 0.1 step size
        for i in 0..1000 {
            {
                let f = (-i) as f64 / 10 as f64;
                let s = format!("{:.1}", f);
                let expected = f64::from_str(&s).unwrap();
                assert_eq!(expected, custom_parse_f64(&s));
            }

            {
                let f = i as f64 / 10 as f64;
                let s = format!("{:.1}", f);
                let expected = f64::from_str(&s).unwrap();
                assert_eq!(expected, custom_parse_f64(&s));
            }
        }
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
    fn test_parse_large_chunks0() {
        let content = create_content(&STATIONS, &TEMPERATURES);
        let rdr = BufReader::with_capacity(64 * 1024, Cursor::new(content.as_bytes()));
        let mut idx: usize = 0;
        parse_large_chunks0(
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

    // #[test]
    // fn custom_parse_f64_simd_works() {
    //     // Verify positive and negative numbers in the range [-99.9, 99.9] with 0.1 step size
    //     for i in 0..1000 {
    //         {
    //             let f = (-i) as f64 / 10 as f64;
    //             let s = format!("{:.1}", f);
    //             let expected = f64::from_str(&s).unwrap();
    //             assert_eq!(expected, custom_parse_f64_simd(&s));
    //         }
    //
    //         {
    //             let f = i as f64 / 10 as f64;
    //             let s = format!("{:.1}", f);
    //             let expected = f64::from_str(&s).unwrap();
    //             assert_eq!(expected, custom_parse_f64_simd(&s));
    //         }
    //     }
    // }
}
