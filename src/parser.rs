
use std::io::prelude::*; use std::{
    fs::File,
    path::{Path, PathBuf},
    time::Instant,
};

use anyhow::Context;
use clap::Parser;
use flate2::{bufread::GzEncoder, Compression};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct Freq {
    token: Box<str>,
    occurrences: u32,
    books: u32,
}

pub fn extract_ngram(input: String) -> anyhow::Result<Vec<Freq>> {
    let t0 = Instant::now();

    let mut storage = Vec::new();
    let mut last_freq = Freq::default();

    let mut count = 0;
    for i in input.lines() {
        count += 1;

        let mut words = i.split('\t');

        let token = words
            .next()
            .context(format!("bad format at line {count}: no token"))?;
        let mut words = words.skip(1);
        let occurrences: u32 = words
            .next()
            .context(format!("bad format at line {count}: no words"))?
            .parse()
            .map_err(|e| anyhow::anyhow!("Error parsing in line {count}:\n{e}"))?;
        let books: u32 = words
            .next()
            .context(format!("bad format at line {count}: no books"))?
            .parse()
            .map_err(|e| anyhow::anyhow!("Error parsing in line {count}:\n{e}"))?;

        if token == last_freq.token.as_ref() {
            last_freq.occurrences += occurrences;
            last_freq.books += books;
        } else {
            storage.push(last_freq);
            last_freq = Freq {
                token: Box::from(token),
                occurrences,
                books,
            };
        }
    }

    for i in storage.iter_mut() {
        if let Some(index) = i.token.rfind('_') {
            // strip _ suffix
            i.token = i.token[..index].into();
        }
    }

    // Stats

    let t1 = t0.elapsed();
    let rate = t1.as_nanos() / count;

    println!(
        "processed {count} lines in {:?}. Rate of {rate}nsec/line",
        t1
    );
    println!(
        "vec is {}KB",
        storage.len() * std::mem::size_of::<Freq>() / (8 * 1024)
    );
    // let t0 = Instant::now();
    // storage.sort_unstable_by_key(|x| x.occurrences);
    // println!("sorting took {:?}", t0.elapsed());

    Ok(storage)
}

pub fn uncompressed_twograms() -> anyhow::Result<()> {
    let mut input: Vec<PathBuf> = WalkDir::new("2grams")
        .into_iter()
        .map(|e| {
            let x = e.unwrap().path().to_owned();
            x
        })
        .collect();
    // input.swap_remove(0);

    let grams = input
        .iter()
        .filter(|x| x.extension().is_none()) // Uncompressed files
        .map(|x| {
            println!("processing {}", x.to_string_lossy());
            x
        })
        .map(|x| std::fs::read_to_string(x).unwrap())
        .map(extract_ngram)
        .filter_map(|x| match x {
            Ok(e) => Some(e),
            Err(e) => {
                eprintln!("{e}");
                None
            }
        })
        .collect::<Vec<_>>();
    // println!("deleting original files...");
    // for i in input.iter() {
    //     std::fs::remove_file(i).unwrap();
    // }

    let t0 = Instant::now();
    println!("done. serializing...");
    let output = postcard::to_allocvec(&grams).unwrap();

    println!("serialized in {:?}. writing to file...", t0.elapsed());
    let t0 = Instant::now();
    let len = output.len() as f64;
    std::fs::write("./2grams.bin", output).unwrap();
    println!(
        "written {}MB in {:?}",
        len / (8 * 1024 * 1024) as f64,
        t0.elapsed()
    );

    Ok(())
}

pub fn uncompress(file: &Path) -> String {
    let file = std::fs::File::open(&file).unwrap();
    let mmap = unsafe { memmap2::MmapOptions::new().map(&file).unwrap() };
    let x: &[u8] = &mmap;
    let mut d = flate2::bufread::GzDecoder::new(x);
    let mut s = String::new();
    d.read_to_string(&mut s).unwrap();
    s
}

pub fn get_filenames(folder: &str) -> impl Iterator<Item = PathBuf> {
    WalkDir::new(folder)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|x| x.file_type().is_file())
        .map(|x| x.path().to_owned())
}

pub fn onegrams() {
    let mut grams1 = get_filenames("1grams")
        .par_bridge()
        .inspect(|x| {
            println!("processing {}", x.to_string_lossy());
        })
        .map(|x| uncompress(&x))
        .flat_map(extract_ngram)
        .flatten()
        .collect::<Vec<Freq>>();

    print!("Done, sorting...");
    let t = Instant::now();
    grams1.par_sort_unstable_by_key(|x| x.occurrences);
    println!("sorting took {:?}", t.elapsed());

    print!("Serializing...");
    let t = Instant::now();
    let output = postcard::to_allocvec(&grams1).unwrap();
    println!("done in {:?}", t.elapsed());
    let t = Instant::now();
    print!("Writing to file...");
    std::fs::write("./1grams.postcard", &output).unwrap();
    println!("done in {:?}", t.elapsed());
    let t = Instant::now();

    print!("Writing to file (gzip)...");
    let f = File::create("./1grams.postcard.gz").unwrap();
    let mut e = flate2::write::GzEncoder::new(f, Compression::default());
    e.write_all(&output).unwrap();
    println!("done in {:?}", t.elapsed());
}

pub fn downloader(files: &str) {
    let mut path = PathBuf::from("2grams");
    for i in files.lines() {
        println!("downloading {i}");
        let res = reqwest::blocking::get(i).unwrap().bytes().unwrap();
        path.push(&i[5..]);
        std::fs::write(&path, res).unwrap();
        path.pop();
    }
}
