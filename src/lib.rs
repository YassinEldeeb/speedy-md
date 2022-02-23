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
enum Type<'a> {
    Heading { value: &'a str, size: usize },
    Paragraph { value: String },
    Blockquotes { value: String },
    List { value: Vec<String> },
    UnorderedLists { value: Vec<String> },
    CodeBlocks { value: String, language: String },
    Links { label: String, url: String },
    Image { label: String, url: String },
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

            result.push(Node {
                value: Type::Heading {
                    value: &line[size..].trim(),
                    size,
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
