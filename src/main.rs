#![allow(unused)]
use std::collections::HashMap;

use clap::Parser;
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
    // onegrams();
    // return;
    let cli = Cli::parse();
    let ptrs = cli.command.iter().map(|command| match command.as_str() {
        "dedup" => dedup,
        "case_insensitive" => dedup_cases,
        _ => unimplemented!()
    });

    let mut freqs = load_1grams("1grams.postcard");
    println!("Found {} words: {}MB", freqs.len(), freqs.len() * std::mem::size_of::<Freq>() / (1024 * 1024));
    for i in ptrs {
        freqs = i(freqs);
    }

    match cli.save.as_deref() {
        Some(path) => write_postcard(path, &freqs),
        _ => {
            for i in freqs.iter().take(100) {
                //println!("{:?}", i);
            }
        }
    }
}
