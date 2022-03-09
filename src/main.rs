use speedy_md::Tokenizer;
use std::{fs, time::Instant};

fn main() {
    let content = fs::read_to_string("dev.md").unwrap();

    let now = Instant::now();

    let res = Tokenizer::new(&content).run();

    let elapsed = now.elapsed();

    println!("Performance: {:?}", elapsed);

    println!("{:#?}", res);
}
