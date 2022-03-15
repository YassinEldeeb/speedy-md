use speedy_md::Lexer;
use std::{fs, time::Instant};

fn main() {
    let content = fs::read_to_string("dev.md").unwrap();

    let now = Instant::now();

    let res = Lexer::new(&content).run();

    let elapsed = now.elapsed();

    println!("Performance: {:?}", elapsed);

    println!("{:#?}", res);
}
