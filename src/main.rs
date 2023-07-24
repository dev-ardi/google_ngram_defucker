use std::{path::PathBuf, time::Instant};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Freq {
    token: Box<str>,
    occurrences: u32,
    specificity: f32,
}

fn extract_ngram(input: String) -> Vec<Freq> {
    let t0 = Instant::now();

    let mut storage = Vec::new();
    let mut last_freq = Freq {
        token: "".into(),
        occurrences: 0,
        specificity: 0.0,
    };

    let mut count = 0;
    for i in input.lines() {
        count += 1;

        let mut words = i.split('\t');

        let token = words.next().unwrap();
        let _ = words.next().unwrap(); // year
        let occurrences: u32 = words.next().unwrap().parse().unwrap();
        let books: u32 = words.next().unwrap().parse().unwrap();

        let specificity = books as f32 / occurrences as f32;

        if token == last_freq.token.as_ref() {
            last_freq.occurrences += occurrences;
            last_freq.specificity += specificity;
        } else {
            storage.push(last_freq);
            last_freq = Freq {
                token: Box::from(token),
                occurrences,
                specificity,
            };
        }
    }
    let t1 = t0.elapsed();
    let rate = t1.as_nanos() / count;
    let seconds = t1.as_secs_f32();

    for i in storage.iter_mut() {
        if let Some(index) = i.token.rfind('_') {
            // strip _ suffix
            i.token = i.token[..index].into();
        }
    }

    println!("processed {count} lines in {seconds}s. Rate of {rate}nsec/line");
    println!("vec is {}KB", storage.len()/ (8*1024));
    let t0 = Instant::now();
    storage.sort_unstable_by_key(|x| x.occurrences);
    println!("sorting took {}s", t0.elapsed().as_secs_f32());

    storage
}

fn uncompressed_twograms() -> anyhow::Result<()> {
    let mut input: Vec<PathBuf> = WalkDir::new("2grams")
        .into_iter()
        .map(|e| {
            let x = e.unwrap().path().to_owned();
            x
        })
        .collect();
    input.swap_remove(0);

    let grams = input
        .iter()
        .filter(|x| x.extension().is_none()) // Uncompressed files
        .map(|x| {
            println!("processing {}", x.to_string_lossy());
            x
        })
        .map(|x| std::fs::read_to_string(x).unwrap())
        .flat_map(extract_ngram)
        .collect::<Vec<_>>();
    println!("deleting original files...");
    for i in input.iter() {
        //std::fs::remove_file(i).unwrap();
    }

    let t0 = Instant::now();
    println!("done. serializing...");
    let output = postcard::to_allocvec(&grams).unwrap();

    println!(
        "serialized in {}s. writing to file...",
        t0.elapsed().as_secs_f32()
    );
    let t0 = Instant::now();
    let len = output.len() as f64;
    std::fs::write("./2grams.bin", output).unwrap();
    println!(
        "written {}MB in {}s",
        len / (8 * 1024 * 1024) as f64,
        t0.elapsed().as_secs_f32()
    );

    Ok(())
}

fn onegrams() -> anyhow::Result<()> {
    let mut input: Vec<PathBuf> = WalkDir::new("1grams")
        .into_iter()
        .map(|e| {
            let x = e.unwrap().path().to_owned();
            x
        })
        .collect();
    input.swap_remove(0);

    let grams1 = input
        .par_iter()
        .map(|x| {
            println!("processing {}", x.to_string_lossy());
            x
        })
        .map(|x| std::fs::read_to_string(x).unwrap())
        .flat_map(extract_ngram)
        .collect::<Vec<_>>();

    println!("done. serializing...");
    let output = postcard::to_allocvec(&grams1).unwrap();
    println!("writing to file...");
    std::fs::write("./1grams.bin", output).unwrap();

    Ok(())
}

fn main() {
    uncompressed_twograms().unwrap();
}

fn downloader(files: &str) {
    let mut path = PathBuf::from("2grams");
    for i in files.lines() {
        println!("downloading {i}");
        let res = reqwest::blocking::get(i).unwrap().bytes().unwrap();
        path.push(&i[5..]);
        std::fs::write(&path, res).unwrap();
        path.pop();
    }
}
