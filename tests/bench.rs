use serde::{Deserialize, Serialize};
use speedy_md::Parser;
use std::{fs, path::Path, time::Instant};
mod utils;

#[derive(Serialize, Deserialize, Debug)]
struct Bench<'a> {
    improvement: String,
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

    let paths = fs::read_dir("./benchmarks").unwrap();
    let mut timestamps = vec![];

    for path in paths {
        let timestamp: u128 = path
            .unwrap()
            .path()
            .as_os_str()
            .to_str()
            .unwrap()
            .split("\\")
            .collect::<Vec<&str>>()[1]
            .replace(".json", "")
            .parse()
            .unwrap();

        timestamps.push(timestamp);
    }

    timestamps.sort();

    // Cleanup old benchmarks
    if timestamps.len() >= 2 {
        // Execlude the last bench file
        for i in &timestamps[..timestamps.len() - 1] {
            fs::remove_file(format!("./benchmarks/{}.json", i))
                .expect("Couldn't remove the last benchmark");
        }
    }

    let improvement = {
        if timestamps.len() >= 1 {
            let last_bench_path = format!("./benchmarks/{}.json", timestamps[timestamps.len() - 1]);

            if Path::new(&last_bench_path).exists() {
                let last_bench: serde_json::Value =
                    serde_json::from_str(&fs::read_to_string(&last_bench_path).unwrap())
                        .expect("JSON was not well-formatted");

                let mut percentage =
                    (average - last_bench.get("average").unwrap().as_f64().unwrap()) / average
                        * 100.0;

                if percentage < 1.5 {
                    percentage = 0.0;
                }
                format!("{}%", percentage)
            } else {
                String::from("0%")
            }
        } else {
            String::from("0%")
        }
    };

    let bench = Bench {
        improvement,
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

    let json = serde_json::to_string_pretty(&bench).unwrap();
    let path = format!("./benchmarks/{}.json", utils::get_unix_timestamp_us());

    fs::write(path, json).unwrap();
}
