#[macro_use(concat_string)]
extern crate concat_string;

use std::{io::Read, time::Instant, vec};

#[derive(PartialEq)]
enum Type {
    Header(usize),
    BlockQuote,
    Paragraph,
    LineBreak,
    Space,
}

pub struct Parser {
    position: usize,
    bytes_result: Option<String>,
}

impl Parser {
    pub fn new() -> Self {
        Parser {}
    }

    pub fn get_html(&self, content: &str) -> String {
        let mut result = String::new();

        let lines: Vec<&str> = content.lines().collect();
        let mut skip = 0_u32;

        for (idx, &line) in lines.iter().enumerate() {
            if skip > 0 {
                skip -= 1;
                continue;
            }
            let parsed = self.parse(line, idx, &lines, &mut skip);
            result.push_str(&parsed);
        }

        result
    }

    fn parse(&self, line: &str, index: usize, lines: &Vec<&str>, skip: &mut u32) -> String {
        match self.identify_line(line) {
            Type::Header(size) => self.parse_header(line, size),
            Type::BlockQuote => self.parse_blockquote(index, lines, skip),
            Type::Paragraph => self.parse_paragraph(index, lines, skip),
            Type::LineBreak => String::new(),
        }
    }

    fn identify_line(&self, line: &str) -> Type {
        if line.is_empty() {
            return Type::LineBreak;
        }

        let char_bytes = line.as_bytes();

        let first_byte = char_bytes[0];

        if first_byte == b'#' {
            let mut size = 0;

            while char_bytes[size] == b'#' {
                size += 1;

                if size >= 7 {
                    break;
                }
            }

            // # Hello -> <h1> 👌
            // #Hello -> <p> 👎
            let space_separator = char_bytes[size];
            if size > 6 || space_separator != b' ' {
                return Type::Paragraph;
            }

            Type::Header(size)
        } else if first_byte == b'>' {
            Type::BlockQuote
        } else {
            Type::Paragraph
        }
    }

    /// Gives you a tuple containing the start and ending positions of a matching condition
    /// (start, end)
    ///
    /// # Example Output
    /// ```
    /// (0, 16)
    /// ```
    fn consume_while<T>(&mut self, bytes: &[u8], condition: T) -> (usize, usize)
    where
        T: Fn(u8) -> bool,
    {
        let start = self.position;
        let mut end = self.position;

        loop {
            if self.position >= bytes.len() {
                break;
            }

            let next_byte = bytes[self.position];

            // Exit the loop if we faced the end of a line
            if condition(next_byte) {
                end += 1;
            } else {
                break;
            }

            self.position += 1;
        }

        (start, end)
    }

    fn parse_header(&self, line: &str, size: usize) -> String {
        let headers = ["<h1>", "<h2>", "<h3>", "<h4>", "<h5>", "<h6>"];
        let closing_headers = ["</h1>", "</h2>", "</h3>", "</h4>", "</h5>", "</h6>"];

        let line = &line[size..];
        let header_index = size - 1;
        self.create_tag(headers[header_index], closing_headers[header_index], line)
    }

    fn parse_paragraph(&self, current_index: usize, lines: &Vec<&str>, skip: &mut u32) -> String {
        let mut index = current_index;
        let mut result = String::from("<p>");

        // Parse sequential paragraph lines
        while index < lines.len() {
            let line = lines[index];

            if self.identify_line(line) == Type::Paragraph && !line.is_empty() {
                let is_not_first_iter = result.len() == 3;

                // if result has already a paragraph line and
                // the previous line doesn't end with a "  "
                // then: Push an empty space to seperate the two lines
                if !is_not_first_iter && !lines[index - 1].ends_with("  ") {
                    result.push(' ');
                }

                result.push_str(line.trim());

                if line.ends_with("  ") {
                    result.push_str("<br>")
                }

                index += 1;
            } else {
                break;
            }
        }

        result.push_str("</p>");

        // Skip the lines we checked above
        *skip += (index - current_index - 1) as u32;

        result
    }

    fn emphasis(&self, line: &str) -> String {
        let mut offset = 0;

        let emph = [
            ("**", "<strong>", "</strong>"),
            ("*", "<em>", "</em>"),
            // ("_", "<em>", "</em>"),
            ("`", "<code>", "</code>"),
            // ("~~", "<del>", "</del>"),
        ];

        // for (patt, tag, closing_tag) in emph {
        //     if line.find(patt).is_none() {
        //         continue;
        //     }

        // }

        // let chars = line.as_bytes();

        // // Chain an empty character so that the last iteration can run to
        // // catch matches at the end of a line.

        // let mut start_pos = None;
        // let mut end_pos = None;
        // let mut matches = 0;

        // let start_pattern = start_pattern.as_bytes();
        // let end_pattern = end_pattern.as_bytes();

        // for (idx, &e) in chars.iter().enumerate() {
        //     let current_pattern = if start_pos.is_none() {
        //         start_pattern
        //     } else {
        //         end_pattern
        //     };

        //     if matches == current_pattern.len() {
        //         if start_pos.is_some() {
        //             end_pos = Some(idx - end_pattern.len());
        //         } else {
        //             start_pos = Some(idx);
        //         }

        //         matches = 0;
        //     }

        //     if e == current_pattern[matches] {
        //         matches += 1;
        //     } else {
        //         matches = 0;
        //     }

        //     if start_pos.is_some() && end_pos.is_some() {
        //         let (start_pos_val, end_pos_val) = (start_pos.unwrap(), end_pos.unwrap());
        //         if end_pos_val - start_pos_val >= 1 {
        //             cb((start_pos.unwrap(), end_pos.unwrap()));
        //         }
        //         start_pos = None;
        //         end_pos = None;
        //     }
        // }

        "".to_string()
    }

    fn capture_pattern<F>(&self, line: &str, start_pattern: &str, end_pattern: &str, mut cb: F)
    where
        F: FnMut((usize, usize)),
    {
        if line.find(start_pattern).is_none() {
            return;
        }

        let chars = line.as_bytes();

        // Chain an empty character so that the last iteration can run to
        // catch matches at the end of a line.

        let mut start_pos = None;
        let mut end_pos = None;
        let mut matches = 0;

        let start_pattern = start_pattern.as_bytes();
        let end_pattern = end_pattern.as_bytes();

        for (idx, &e) in chars.iter().enumerate() {
            let current_pattern = if start_pos.is_none() {
                start_pattern
            } else {
                end_pattern
            };

            if matches == current_pattern.len() {
                if start_pos.is_some() {
                    end_pos = Some(idx - end_pattern.len());
                } else {
                    start_pos = Some(idx);
                }

                matches = 0;
            }

            if e == current_pattern[matches] {
                matches += 1;
            } else {
                matches = 0;
            }

            if start_pos.is_some() && end_pos.is_some() {
                let (start_pos_val, end_pos_val) = (start_pos.unwrap(), end_pos.unwrap());
                if end_pos_val - start_pos_val >= 1 {
                    cb((start_pos.unwrap(), end_pos.unwrap()));
                }
                start_pos = None;
                end_pos = None;
            }
        }
    }

    fn capture_simple_pattern<F>(&self, line: &str, pattern: &str, cb: F)
    where
        F: FnMut((usize, usize)),
    {
        self.capture_pattern(line, pattern, pattern, cb)
    }

    fn create_tag(&self, tag: &str, closing_tag: &str, content: &str) -> String {
        if content.is_empty() {
            String::new()
        } else {
            concat_string!(tag, content.trim(), closing_tag)
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn blackquote() {
//         let parser = Parser::new();
//         let mut skip = 0;

//         let blackquote = parser.parse_blockquote(
//             0,
//             &vec!["> Yassin Said", ">", "> That he's so dumb"],
//             &mut skip,
//         );

//         assert_eq!(skip, 2);
//         assert_eq!(
//             blackquote,
//             "<blockquote><p>Yassin Said</p><p>That he's so dumb</p></blockquote>"
//         );

//         let mut skip = 0;
//         let blackquote =
//             parser.parse_blockquote(0, &vec!["> Yassin Said", "> That he's so dumb"], &mut skip);

//         assert_eq!(skip, 1);
//         assert_eq!(
//             blackquote,
//             "<blockquote><p>Yassin Said That he's so dumb</p></blockquote>"
//         );
//     }

//     #[test]
//     fn header() {
//         let parser = Parser::new();

//         let header = parser.parse_header("# Hey", 1);
//         assert_eq!(header, "<h1>Hey</h1>");

//         let header = parser.parse_header("#### Hey", 4);
//         assert_eq!(header, "<h4>Hey</h4>");

//         let header = parser.parse_header("###### Hey", 6);
//         assert_eq!(header, "<h6>Hey</h6>");

//         let header = parser.parse_header("## Hey", 2);
//         assert_eq!(header, "<h2>Hey</h2>");
//     }

//     #[test]
//     fn paragraph() {
//         let parser = Parser::new();

//         let mut skip = 0;
//         let paragraph = parser.parse_paragraph(0, &vec!["  Hello World  "], &mut skip);
//         assert_eq!(paragraph, "<p>Hello World<br></p>");

//         let mut skip = 0;
//         let paragraph = parser.parse_paragraph(0, &vec!["I'm Yassin", "", "WoW"], &mut skip);
//         assert_eq!(paragraph, "<p>I'm Yassin</p>");

//         let mut skip = 0;
//         let paragraph = parser.parse_paragraph(2, &vec!["I'm Yassin", "", "WoW"], &mut skip);
//         assert_eq!(paragraph, "<p>WoW</p>");

//         let mut skip = 0;
//         let paragraph = parser.parse_paragraph(0, &vec!["#Hello", "World"], &mut skip);
//         assert_eq!(paragraph, "<p>#Hello World</p>");
//     }

//     #[test]
//     fn emphasis() {
//         let parser = Parser::new();

//         let emphasised = parser.emphasis("**H**ello, I'm *Yassin* not ~~Husien~~. I'm a `coder`");
//         assert_eq!(
//             emphasised,
//             "<b>H</b>ello, I'm <em>Yassin</em> not <del>Husien</del>. I'm a <code>coder</code>"
//         );

//         let emphasised = parser.emphasis("Can emph las**t**");
//         assert_eq!(emphasised, "Can emph las<b>t</b>");

//         let emphasised = parser.emphasis("Can emph ***~~nested~~***");
//         assert_eq!(emphasised, "Can emph <b><em><del>nested</del></b></em>");

//         let emphasised = parser.emphasis("*C*an emph first");
//         assert_eq!(emphasised, "<em>C</em>an emph first");
//     }

//     #[test]
//     fn capture_comlex_pattern() {
//         let parser = Parser::new();

//         let captured = parser.capture_pattern("![link]", "![", "]");
//         assert_eq!(captured, vec![(2, 6)]);

//         let captured = parser.capture_pattern("*&<link>~!", "*&<", ">~!");
//         assert_eq!(captured, vec![(3, 7)]);

//         let captured = parser.capture_pattern("^(special) something ^(or) no^(t)", "^(", ")");
//         assert_eq!(captured, vec![(2, 9), (23, 25), (31, 32)]);
//     }

//     #[test]
//     fn capture_simple_pattern() {
//         let parser = Parser::new();

//         let captured = parser.capture_simple_pattern("*&This is a simple line*& *&l*&", "*&");
//         assert_eq!(captured, vec![(2, 23), (28, 29)]);

//         let captured = parser.capture_simple_pattern("**Th**is **is** a si**mple line**", "**");
//         assert_eq!(captured, vec![(2, 4), (11, 13), (22, 31)]);

//         let captured = parser.capture_simple_pattern("Last lette`r`", "`");
//         assert_eq!(captured, vec![(11, 12)]);

//         let captured = parser.capture_simple_pattern("`F`irst letter", "`");
//         assert_eq!(captured, vec![(1, 2)]);
//     }

//     #[test]
//     fn create_tag() {
//         let parser = Parser::new();

//         let tag = parser.create_tag("<code>", "</code>", "console.log(\"Hello\")");
//         assert_eq!(tag, String::from("<code>console.log(\"Hello\")</code>"));

//         let tag = parser.create_tag("<p>", "</p>", "  ~~H~~ello ");
//         assert_eq!(tag, String::from("<p><del>H</del>ello</p>"));
//     }
// }
