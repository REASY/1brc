use brc_core::{improved_impl_v3, sort_result, State};
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
use std::str::FromStr;
use std::thread;
use std::time::Instant;

const THREAD_STACK_SIZE: usize = 10 * 1024 * 1024;

const BUF_READER_CAPACITY: usize = 10 * 1024 * 1024;

fn main() {
    let instant = Instant::now();

    let path = std::env::args().skip(1).next().unwrap_or_else(|| {
        "C:/repos/github/REASY/1brc/brc-core/test_resources/sample.txt".to_owned()
    });
    let cores: usize = std::env::args()
        .skip(2)
        .next()
        .map(|c| usize::from_str(c.as_str()).unwrap())
        .unwrap_or_else(|| thread::available_parallelism().unwrap().into());

    let file = File::open(&path).unwrap();
    let file_length = file.metadata().unwrap().len() as usize;

    let xs = if cores <= 1 {
        let rdr = BufReader::with_capacity(10 * 1024 * 1024, File::open(&path).unwrap());
        vec![improved_impl_v3(rdr, 0, (file_length - 1) as u64, false)]
    } else {
        let chunks = get_chunks(cores, file);
        let threads: Vec<_> = chunks
            .iter()
            .map(|(s, e)| {
                let start = *s as u64;
                let end_inclusive = *e as u64;
                let path = path.clone();
                thread::Builder::new()
                    .stack_size(THREAD_STACK_SIZE)
                    .spawn(move || {
                        let rdr = BufReader::with_capacity(
                            BUF_READER_CAPACITY,
                            File::open(&path).unwrap(),
                        );
                        improved_impl_v3(rdr, start, end_inclusive, false)
                    })
                    .unwrap()
            })
            .collect();
        let mut r: Vec<Vec<(String, State)>> = Vec::with_capacity(cores);
        for t in threads {
            r.push(t.join().unwrap());
        }
        r
    };

    let mut hs: hashbrown::HashMap<String, State> = hashbrown::HashMap::new();
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

    let output = prepare_output(&mut final_result);

    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(output.as_bytes()).unwrap();

    let avg_processing_througput = (file_length as f64 / 1024.0f64 / 1024.0f64)
        / (instant.elapsed().as_millis() as f64 / 1000.0f64);
    println!(
        "Processed in {} ms, avg_processing_througput: {:.4} MBytes/s",
        instant.elapsed().as_millis(),
        avg_processing_througput
    );
}

fn prepare_output(final_result: &mut Vec<(String, State)>) -> String {
    let mut res: String = String::new();
    res.push_str("{{");
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
    res.push_str("}}");
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
    println!("For {cores} cores prepared {} chunks, chunk_size: {chunk_size}, file_length: {file_length}", chunks.len());
    chunks
}
