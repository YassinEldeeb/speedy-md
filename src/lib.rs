#[derive(Debug)]
struct Position {
    line: usize,
    cursor: usize,
}

#[derive(Debug)]
pub struct Node<'a> {
    value: Type<'a>,
    boundaries: (Position, Position),
}

#[derive(Debug)]
enum Emphasis {
    Bold {
        start_cursor: usize,
        end_cursor: usize,
    },
    Italic {
        start_cursor: usize,
        end_cursor: usize,
    },
    Code {
        start_cursor: usize,
        end_cursor: usize,
    },
}

#[derive(Debug)]
enum Type<'a> {
    Heading {
        value: &'a str,
        size: usize,
        emphasis: Vec<Emphasis>,
    },
    Paragraph {
        value: String,
    },
    Blockquotes {
        value: String,
    },
    List {
        value: Vec<String>,
    },
    UnorderedLists {
        value: Vec<String>,
    },
    CodeBlocks {
        value: String,
        language: String,
    },
    Links {
        label: String,
        url: String,
    },
    Image {
        label: String,
        url: String,
    },
}

pub fn parse<'a>(content: &'a str) -> Vec<Node<'a>> {
    let mut result: Vec<Node> = Vec::new();

    for (idx, line) in content.lines().enumerate() {
        let line = line.trim();

        if line.starts_with('#') {
            let mut size = 0;

            for e in line.chars() {
                if e == '#' {
                    size += 1;
                } else {
                    break;
                }
            }

            let mut emph = Vec::new();

            emphasis(line, '*', 2, |start_cursor, end_cursor| {
                emph.push(Emphasis::Bold {
                    start_cursor,
                    end_cursor,
                });
            });

            emphasis(line, '*', 1, |start_cursor, end_cursor| {
                emph.push(Emphasis::Italic {
                    start_cursor,
                    end_cursor,
                });
            });
            emphasis(line, '`', 1, |start_cursor, end_cursor| {
                emph.push(Emphasis::Code {
                    start_cursor,
                    end_cursor,
                });
            });

            result.push(Node {
                value: Type::Heading {
                    value: &line[size..].trim(),
                    size,
                    emphasis: emph,
                },
                boundaries: (
                    Position {
                        line: idx,
                        cursor: 0,
                    },
                    Position {
                        line: idx,
                        cursor: line.len(),
                    },
                ),
            })
        }
    }

    result
}

fn emphasis<T>(line: &str, identifier: char, repeat: usize, mut cb: T)
where
    T: FnMut(usize, usize),
{
    let mut identifier_repeatition = 0;
    let mut start_pos = None;
    let mut end_pos = None;

    for (idx, i) in line.chars().enumerate() {
        if i == identifier {
            identifier_repeatition += 1;
        } else {
            identifier_repeatition = 0;
        }

        if identifier_repeatition == repeat {
            if start_pos.is_some() {
                // Add 1 so that execlusive range can contain the last char
                end_pos = Some(idx - repeat + 1);
                identifier_repeatition = 0;
            } else {
                start_pos = Some(idx + 1);
                identifier_repeatition = 0;
            }
        }

        if start_pos.is_some() && end_pos.is_some() {
            let start = start_pos.unwrap();
            let end = end_pos.unwrap();

            if end - start > 1 {
                cb(start, end);
            }
            start_pos = None;
            end_pos = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use crate::parse;

    #[test]
    fn benchmark() {
        use pulldown_cmark::{html, Options, Parser};

        let now = Instant::now();

        let markdown_input = "# ***I'm*** super **C**hunky

## _I_'m a `v`ery `big`
";

        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        let parser = Parser::new_ext(markdown_input, options);

        // Write to String buffer.
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        let elapsed = now.elapsed();

        println!("Pulldown Parser {:?}", elapsed);

        //

        let now2 = Instant::now();

        let res = parse(markdown_input);

        let elapsed2 = now2.elapsed();

        println!("Mine {:?}", elapsed2);

        let diff = elapsed.as_micros() / elapsed2.as_micros();

        println!("Faster by {} times!", diff);
        assert!(diff > 3);
    }
}
