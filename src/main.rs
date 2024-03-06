#![allow(unused)]
use clap::Parser;
use parser::{onegrams, Freq};

mod parser;

#[derive(Parser)]
struct Cli {
    /// this flag to call parser for 1grams, 2grams etc
    #[clap(short, long)]
    parse: Option<String>,
}

fn analyze_1grams() {
    let f = std::fs::read("1grams.postcard").unwrap();
    let grams: Vec<Freq> = postcard::from_bytes(&f).unwrap();
    println!("{:?}", grams[..100]);
}

fn main() {
    let cli = Cli::parse();
    match cli.parse.as_deref() {
        Some("1grams") => onegrams(),
        Some(_) => todo!(),
        None => analyze_1grams(),
    };
}
