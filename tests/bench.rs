use serde::{Deserialize, Serialize};
use std::{fs, path::Path, time::Instant};
mod utils;
use speedy_md::Parser;

#[derive(Serialize, Deserialize, Debug)]
struct BenchInfo {
    parser: String,
    measurement_unit: String,
    no_of_lines: usize,
    content_size_in_bytes: usize,
}
#[derive(Serialize, Deserialize, Debug)]
struct Bench {
    improvement: String,
    info: BenchInfo,
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
    let parser_name = String::from("speedy-md");
    let parser = Parser::new();

    let content =
        fs::read_to_string("./tests/fixtures/bench.md").expect("`./bench.md` has been deleted!");

    let mut results = vec![];
    let num_of_iterations = 50;

    for i in 0..num_of_iterations {
        let now = Instant::now();
        let res = parser.get_html(&content);

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

    let is_ci = ci_info::is_ci();

    // Only write results if not running in CI
    if !is_ci {
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
        if timestamps.len() >= 10 {
            // Execlude the last bench file
            for i in &timestamps[..timestamps.len() - 1] {
                fs::remove_file(format!("./benchmarks/{}.json", i))
                    .expect("Couldn't remove the last benchmark");
            }
        }

        let mut perc = 0.0;
        let improvement = {
            if timestamps.len() >= 1 {
                let last_bench_path =
                    format!("./benchmarks/{}.json", timestamps[timestamps.len() - 1]);

                if Path::new(&last_bench_path).exists() {
                    let last_bench: serde_json::Value =
                        serde_json::from_str(&fs::read_to_string(&last_bench_path).unwrap())
                            .expect("JSON was not well-formatted!");

                    let percentage =
                        (average - last_bench.get("average").unwrap().as_f64().unwrap()) / average
                            * 100.0;

                    perc = -percentage;
                    // Threshold = 1.5%
                    if percentage < 1.5 && percentage > -1.5 {
                        format!("{}%", 0)
                    } else {
                        format!(
                            "{}{:.2}%",
                            if -percentage > 0.0 { "+" } else { "" },
                            -percentage
                        )
                    }
                } else {
                    String::from("0%")
                }
            } else {
                String::from("0%")
            }
        };

        let bench = Bench {
            improvement,
            info: BenchInfo {
                parser: parser_name,
                measurement_unit: String::from("ms"),
                no_of_lines: content.lines().collect::<Vec<&str>>().len(),
                content_size_in_bytes: content.bytes().len(),
            },
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

        // If the speed is down by 10% than the last bench,
        // Then It's a failure!
        if perc < -10.0 {
            panic!("That's really  slow!");
        }
    }
}
