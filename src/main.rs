#![allow(unused)]
use std::collections::HashMap;

use clap::Parser;
use compact_str::CompactString;
use itertools::Itertools;
use parser::{dedup, onegrams, write_postcard, Freq};
use serde::{Deserialize, Serialize};

mod parser;

#[derive(Parser)]
struct Cli {
    /// this flag to call parser for 1grams, 2grams etc
    // #[clap(short, long)]
    command: Vec<String>,

    #[clap(short, long)]
    save: Option<String>,
}

fn load_1grams(path: &str) -> Vec<Freq> {
    let f = std::fs::read(path).unwrap();
    postcard::from_bytes(&f).unwrap()
}

fn dedup_file() {
    let fs = dedup(load_1grams("1grams.postcard"));
    write_postcard("1grams.postcard", &fs)
}

fn print_contents() {
    let fs = load_1grams("1grams.postcard");
    for i in fs.iter().rev().map(|x| x.token.clone()).take(200) {
        println!("{i}")
    }
}
fn dedup_cases(mut freqs: Vec<Freq>) -> Vec<Freq> {
    for i in freqs.iter_mut() {
        i.token = i.token.to_lowercase().into();
    }
    dedup(freqs)
}

type Filter = fn(Vec<Freq>) -> Vec<Freq>;
fn main() {
    let freqs = load_1grams("1grams_compact_str.postcard");
    let len1 = freqs.len();
    println!("Found {} words: {}MB", len1, len1 * std::mem::size_of::<Freq>() / (1024 * 1024));
    let freqs = dedup(freqs);
    println!("deleted {} keys", len1 - freqs.len());
}
