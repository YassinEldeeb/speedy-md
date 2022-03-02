use speedy_md::Parser;
use std::{fs, time::Instant};

fn main() {
    let content = fs::read_to_string("dev.md").unwrap();

    let now = Instant::now();
    let parser = Parser::new();

    let res = parser.get_html(content);

    let elapsed = now.elapsed();

    println!("Performance: {:?}", elapsed);

    println!("{:#?}", res);
}
