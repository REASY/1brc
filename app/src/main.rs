use brc_core::{improved_impl_v3, sort_result, State};
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
use std::str::FromStr;
use std::time::Instant;

fn main() {
    let instant = Instant::now();
    let cores: usize = std::env::var_os("CORES")
        .map(|c| usize::from_str(c.to_str().unwrap()).unwrap())
        .unwrap_or(std::thread::available_parallelism().unwrap().into());
    let path = std::env::args().skip(1).next().unwrap_or_else(|| {
        "C:/repos/github/REASY/1brc/brc-core/test_resources/sample.txt".to_owned()
    });
    let chunks = get_chunks(cores, &path);
    let mut hs: hashbrown::HashMap<String, State> = hashbrown::HashMap::new();
    for (s, e) in chunks {
        let rdr = BufReader::with_capacity(10 * 1024 * 1024, File::open(&path).unwrap());
        let r = improved_impl_v3(rdr, s as u64, e as u64, false);
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

    println!("Elapsed {} ms", instant.elapsed().as_millis());
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

fn get_chunks(cores: usize, path: &String) -> Vec<(usize, usize)> {
    let file = File::open(&path).unwrap();
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
