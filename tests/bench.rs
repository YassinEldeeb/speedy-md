use serde::{Deserialize, Serialize};
use speedy_md::Parser;
use std::{fs, time::Instant};

#[derive(Serialize, Deserialize, Debug)]
struct Bench<'a> {
    measurement_unit: &'a str,
    average: f64,
    max: f64,
    min: f64,
    num_of_iterations: u128,
    iterations: Vec<Iteration>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Iteration {
    index: u128,
    ms: f64,
}
#[test]
fn bench() {
    let parser = Parser::new();

    let content =
        fs::read_to_string("./tests/fixtures/bench.md").expect("`./bench.md` has been deleted!");

    let mut results = vec![];
    let num_of_iterations = 100;

    for i in 0..num_of_iterations {
        let now = Instant::now();
        let _ = parser.get_html(content.clone());
        let elapsed = now.elapsed();

        results.push((i, elapsed.as_micros()));
    }

    // In Micro-seconds
    let sum: u128 = results.iter().map(|a| a.1).sum();
    let min = results.iter().map(|a| a.1).min().unwrap();
    let max = results.iter().map(|a| a.1).max().unwrap();

    // In Milli-seconds
    let average = (sum as f64 / num_of_iterations as f64) / 1000.0;
    let min = min as f64 / 1000.0;
    let max = max as f64 / 1000.0;

    let bench = Bench {
        measurement_unit: "ms",
        average,
        max,
        min,
        iterations: results
            .iter()
            .map(|(idx, micros)| Iteration {
                index: *idx,
                ms: *micros as f64 / 1000.0,
            })
            .collect(),
        num_of_iterations,
    };

    let json = serde_json::to_string(&bench).unwrap();

    fs::write("./bench.json", json).unwrap();
}
