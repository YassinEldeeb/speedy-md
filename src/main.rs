use speedy_md::parse;
use std::{fs, io::Read, time::Instant};

fn main() {
    let content = fs::read_to_string("test.md").unwrap();

    let now = Instant::now();
    let res = parse(&content);

    let elapsed = now.elapsed();

    println!("Performance: {:?}", elapsed);

    // println!("{:#?}", res);
}
