use brc_core::{
    naive_line_by_line, naive_line_by_line_dummy, naive_line_by_line_v2, parse_large_chunks,
    parse_large_chunks_dummy, parse_large_chunks_simd, parse_large_chunks_simd_dummy,
    parse_large_chunks_simd_v1, parse_large_chunks_v1, parse_large_chunks_v2, sort_result,
    StateF64,
};
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
use std::str::FromStr;
use std::thread;
use std::time::Instant;

/// The capacity of BufReader to improve reading
const BUF_READER_CAPACITY: usize = 64 * 1024 * 1024;

const DEFAULT_IMPL: &str = "naive_line_by_line";

fn main() {
    let instant = Instant::now();
    let path = std::env::args()
        .skip(1)
        .next()
        .unwrap_or_else(|| "brc-core/test_resources/sample.txt".to_owned());
    let cores: usize = std::env::args()
        .skip(2)
        .next()
        .map(|c| usize::from_str(c.as_str()).unwrap())
        .unwrap_or_else(|| thread::available_parallelism().unwrap().into());

    let method: String = std::env::args()
        .skip(3)
        .next()
        .map(|c| c.clone())
        .unwrap_or_else(|| DEFAULT_IMPL.to_string());

    let func: fn(BufReader<_>, u64, u64, bool) -> Vec<(String, StateF64)> = match method.as_str() {
        "naive_line_by_line_dummy" => naive_line_by_line_dummy,
        DEFAULT_IMPL => naive_line_by_line,
        "naive_line_by_line_v2" => naive_line_by_line_v2,
        "parse_large_chunks_dummy" => parse_large_chunks_dummy,
        "parse_large_chunks" => parse_large_chunks,
        "parse_large_chunks_v1" => parse_large_chunks_v1,
        "parse_large_chunks_simd_dummy" => parse_large_chunks_simd_dummy,
        "parse_large_chunks_simd" => parse_large_chunks_simd,
        "parse_large_chunks_simd_v1" => parse_large_chunks_simd_v1,
        "parse_large_chunks_v2" => parse_large_chunks_v2,
        x => panic!("{}", x),
    };

    let file = File::open(&path).unwrap();
    let file_length = file.metadata().unwrap().len() as usize;

    let xs = if cores <= 1 {
        let rdr = BufReader::with_capacity(10 * 1024 * 1024, File::open(&path).unwrap());
        vec![func(rdr, 0, (file_length - 1) as u64, true)]
    } else {
        //
        let chunks = get_chunks(cores, file);
        let threads: Vec<_> = chunks
            .iter()
            .map(|(s, e)| {
                let start = *s as u64;
                let end_inclusive = *e as u64;
                let path = path.clone();
                thread::Builder::new()
                    .spawn(move || {
                        let rdr = BufReader::with_capacity(
                            BUF_READER_CAPACITY,
                            File::open(&path).unwrap(),
                        );
                        func(rdr, start, end_inclusive, false)
                    })
                    .unwrap()
            })
            .collect();
        let mut r: Vec<Vec<(String, StateF64)>> = Vec::with_capacity(cores);
        for t in threads {
            r.push(t.join().unwrap());
        }
        r
    };

    // Build the final hashmap by merging all the measurements for the same location
    let mut hs: hashbrown::HashMap<String, StateF64> = hashbrown::HashMap::new();
    for r in xs {
        for (k, s) in r {
            match hs.get_mut(k.as_str()) {
                None => {
                    hs.insert(k, s);
                }
                Some(prev) => {
                    prev.merge(&s);
                }
            }
        }
    }
    let mut final_result = hs.into_iter().collect();
    sort_result(&mut final_result);

    // Prepare result and write to console
    let output = prepare_output(&mut final_result);
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(output.as_bytes()).unwrap();

    // Write some stats
    let file_length_mbytes = file_length as f64 / 1024.0f64 / 1024.0f64;
    let elapsed_secs = instant.elapsed().as_millis() as f64 / 1000.0f64;
    let avg_processing_throughput = file_length_mbytes / elapsed_secs;
    eprintln!(
        "Processed using `{method}` in {} ms, avg_processing_throughput: {:.4} MBytes/s",
        instant.elapsed().as_millis(),
        avg_processing_throughput
    );
}

fn prepare_output(final_result: &mut Vec<(String, StateF64)>) -> String {
    let mut res: String = String::new();
    res.push_str("{");
    for (i, (name, state)) in final_result.iter().enumerate() {
        if i == 0 {
            res.push_str(name.as_str());
            res.push_str("=");
            res.push_str(&state.to_string());
        } else {
            res.push_str(", ");
            res.push_str(name.as_str());
            res.push_str("=");
            res.push_str(&state.to_string());
        }
    }
    res.push_str("}");
    res.push_str("\n");
    res
}

fn get_chunks(cores: usize, file: File) -> Vec<(usize, usize)> {
    let file_length = file.metadata().unwrap().len() as usize;
    let mut rdr = BufReader::with_capacity(1024 * 1024, file);
    let chunk_size = file_length / cores;
    let mut chunks: Vec<(usize, usize)> = vec![];
    let mut start = 0;

    let mut buf = [0_u8; 512];
    for _ in 0..cores {
        let end = (start + chunk_size).min(file_length - 1);
        rdr.seek(SeekFrom::Start(end as u64)).unwrap();

        let read_bytes = rdr.read(&mut buf).unwrap();
        assert_ne!(0, read_bytes, "start: {start}, end: {end}");

        // We move forward to find the closes new line to simplify reading per chunk - a chunk is always complete, it will have full line
        let valid_buf = &buf[0..read_bytes];
        let mut i: usize = 0;
        while i < valid_buf.len() && valid_buf[i] != 0xA {
            i += 1;
        }
        assert_ne!(
            i,
            valid_buf.len(),
            "Could not find 0xA in the buffer, something is wrong..."
        );
        let fixed_end = end + i;
        chunks.push((start, fixed_end));
        start = fixed_end + 1;
    }
    eprintln!("For {cores} cores prepared {} chunks, chunk_size: {chunk_size}, file_length: {file_length}", chunks.len());
    chunks
}
