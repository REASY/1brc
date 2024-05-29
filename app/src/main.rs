// #![feature(slice_internals)]

use brc_core::{improved_impl_v2, improved_impl_v3, sort_result, State};
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::time::Instant;
use std::{fmt::Display, fs::File};

fn merge<'a>(a: &mut hashbrown::HashMap<String, State>, b: &Vec<(String, State)>) {
    for (k, v) in b {
        a.entry(k.clone()).or_default().merge(v);
    }
}

fn main() {
    let instant = Instant::now();
    let cores: usize = 16; //std::thread::available_parallelism().unwrap().into();
    println!("cores: {cores}");
    println!("cores: {cores}");
    let path = match std::env::args().skip(1).next() {
        Some(path) => path,
        None => "C:/repos/github/REASY/1brc/measurements.txt".to_owned(),
    };
    let file = File::open(&path).unwrap();
    let file_length = file.metadata().unwrap().len() as usize;
    let mut rdr = BufReader::with_capacity(1024 * 1024, file);

    let chunk_size = file_length / cores;
    let mut chunks: Vec<(usize, usize)> = vec![];
    let mut start = 0;

    let mut buf = [0_u8; 512];

    println!("chunk_size: {chunk_size}, file_length: {file_length}");

    for _ in 0..cores {
        let end = (start + chunk_size).min(file_length - 1);
        rdr.seek(SeekFrom::Start(end as u64)).unwrap();
        let read_bytes = rdr.read(&mut buf).unwrap();
        if read_bytes == 0 {
            panic!("start: {start}, end: {end}");
        }
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
        let end = end + i;
        println!("New range: ({start}, {end})");

        chunks.push((start, end));
        start = end + 1;
    }
    println!("{:?}", chunks);

    let mut hs: hashbrown::HashMap<String, State> = hashbrown::HashMap::new();
    for (s, e) in chunks {
        let r = improved_impl_v3(
            BufReader::with_capacity(10 * 1024 * 1024, File::open(&path).unwrap()),
            s as u64,
            e as u64,
            false,
        );
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

    for (k, v) in final_result {
        println!("{k}: {v}");
    }

    println!("Elapsed {} ms", instant.elapsed().as_millis());

    // let mut buf = String::with_capacity(200);
    // let mut buf = [0_u8; 512 * 1024];
    // let mut sum: usize = 0;
    // let mut iters: usize = 0;
    // let mut station_name_vec: Vec<u8> = Vec::with_capacity(200);
    //
    // let mut hs: std::collections::HashMap<String, State> = std::collections::HashMap::new();
    //
    // let mut s: String = String::new();
    //
    // loop {
    //     match rdr.read_line(&mut s) {
    //         Ok(0) => break,
    //         Ok(n) => {
    //             // println!("Read {n} bytes");
    //             let buf_slice = s.as_bytes();
    //             sum += n;
    //             let mut i: usize = 0;
    //             while i < n {
    //                 let b = buf_slice[i];
    //                 if b == b';' {
    //                     let station_name =  unsafe {
    //                         std::str::from_utf8_unchecked(station_name_vec.as_slice())
    //                     };
    //
    //                     let mut j: usize = i + 1;
    //                     let mut has_ending: bool = false;
    //                     while (j < n) {
    //                         if buf_slice[j] == 0xA {
    //                             has_ending = true;
    //                             let float_slice = &buf_slice[i + 1 .. j];
    //                             let float_str = unsafe {
    //                                 std::str::from_utf8_unchecked(float_slice)
    //                             };
    //                             // println!("float_str: {}", float_str);
    //                             let d = f64::from_str(float_str).unwrap();
    //                             match hs.get_mut(station_name) {
    //                                 None => {
    //                                     let mut s = State::default();
    //                                     s.update(d);
    //                                     hs.insert(station_name.to_string(), s);
    //                                 }
    //                                 Some(prev) => {
    //                                     prev.update(d)
    //                                 }
    //                             }
    //
    //                             // println!("station_name: {station_name}, d: {}", d);
    //                             break;
    //                         }
    //                         j += 1;
    //                     }
    //                     station_name_vec.clear();
    //
    //                     i = j;
    //                 }
    //                 else {
    //                     station_name_vec.push(b);
    //                 }
    //
    //                 i += 1;
    //             }
    //             iters += 1;
    //             s.clear();
    //         }
    //         Err(_) => {}
    //     }
    // }
    //
    // for (k, v) in hs {
    //     println!("{k}: {v}");
    // }
    //
    //
    // let mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
    //
    // let chunk_size = mmap.len() / cores;
    // let mut chunks: Vec<(usize, usize)> = vec![];
    // let mut start = 0;
    // for _ in 0..cores {
    //     let end = (start + chunk_size).min(mmap.len());
    //     let next_new_line = match memchr::memchr(b'\n', &mmap[end..]) {
    //         Some(v) => v,
    //         None => {
    //             assert_eq!(end, mmap.len());
    //             0
    //         }
    //     };
    //     let end = end + next_new_line;
    //     chunks.push((start, end));
    //     start = end + 1;
    // }
    // let parts: Vec<_> = chunks
    //     .par_iter()
    //     .map(|r| solve_for_part(*r, &mmap))
    //     .collect();
    //
    // let state: HashMap<&BStr, State> = parts.into_iter().fold(Default::default(), |mut a, b| {
    //     merge(&mut a, &b);
    //     a
    // });
    //
    // let mut all: Vec<_> = state.into_iter().collect();
    // all.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    // let mut res: String = String::new();
    // res.push_str("{{");
    // for (i, (name, state)) in all.into_iter().enumerate() {
    //     if i == 0 {
    //         res.push_str(name.to_str().unwrap());
    //         res.push_str("=");
    //         res.push_str(&state.to_string());
    //     } else {
    //         res.push_str(", ");
    //         res.push_str(name.to_str().unwrap());
    //         res.push_str("=");
    //         res.push_str(&state.to_string());
    //     }
    // }
    // res.push_str("}}");
    //
    // let stdout = io::stdout();
    // let mut handle = stdout.lock();
    // handle.write_all(res.as_bytes()).unwrap();
    // println!();
    // println!("Sum: {sum}, iters: {iters}, elapsed {} ms", instant.elapsed().as_millis())
}
