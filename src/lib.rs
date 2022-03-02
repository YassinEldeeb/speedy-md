use std::vec;

#[derive(PartialEq)]
enum Type {
    Header,
    BlockQuote,
    Paragraph,
    LineBreak,
}
pub struct Parser {}

impl Parser {
    pub fn new() -> Self {
        Parser {}
    }

    pub fn get_html(&self, content: String) -> String {
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
            Type::Header => self.parse_header(index, &lines, skip),
            Type::BlockQuote => self.parse_blockquote(index, lines, skip),
            Type::Paragraph => self.parse_paragraph(index, lines, skip),
            Type::LineBreak => String::new(),
        }
    }

    fn identify_line(&self, line: &str) -> Type {
        if line.starts_with('#') {
            Type::Header
        } else if line.starts_with('>') {
            Type::BlockQuote
        } else if line.is_empty() {
            Type::LineBreak
        } else {
            Type::Paragraph
        }
    }

    fn parse_blockquote(&self, current_index: usize, lines: &Vec<&str>, skip: &mut u32) -> String {
        let mut index = current_index;
        let mut result = String::from("<blockquote>");

        // Parse sequential blockquotes lines
        while index < lines.len() {
            let line = lines[index];

            if self.identify_line(line) == Type::BlockQuote && !line.is_empty() {
                index += 1;
            } else {
                break;
            }
        }

        let mut skip_inner = 0;
        let diff = index - current_index;
        for i in 0..diff {
            if skip_inner > 0 {
                skip_inner -= 1;
                continue;
            }

            let new_lines: Vec<&str> = lines[current_index..current_index + diff]
                .iter()
                .map(|&l| l[1..].trim_start())
                .collect();

            result.push_str(&self.parse(new_lines[i], i, &new_lines, &mut skip_inner));
        }

        result.push_str("</blockquote>");

        // Skip the lines we checked above
        *skip += (index - current_index - 1) as u32;

        result
    }

    fn parse_header(&self, index: usize, lines: &Vec<&str>, skip: &mut u32) -> String {
        let line = lines[index];
        let mut size = 0;

        let mut chars = line.chars();
        while chars.next() == Some('#') {
            size += 1;
        }

        // # Hello -> " " 👌
        // #Hello -> "H" 👎
        let space_separator = &line[size..size + 1];
        if size > 6 || space_separator != " " {
            return self.parse_paragraph(index, lines, skip);
        }

        let line = &line[size..];
        self.create_tag(&format!("h{}", size), line)
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
        let emph = vec![
            ("**", "b"),
            ("*", "em"),
            ("_", "em"),
            ("`", "code"),
            ("~~", "del"),
        ];

        let mut formatted_line = String::from(line);

        fn format(
            mut line: String,
            res: Vec<(usize, usize)>,
            pattern: &str,
            tag_name: &str,
        ) -> String {
            let mut offset = 0;

            // # Logic!

            // He**l**lo **Y**
            // (4, 5)
            // (12, 13)
            // He<b>l</b>lo **Y**
            // (12 + 3, 13 + 3)
            // (15, 16)

            // *I'*m *super*
            // (1, 3)
            // (7, 12)
            // <em>I'</em>m *super*
            // (7 + 7, 12 + 7)
            // (14, 19)

            // ~~I'~~m ~~super~~
            // (2, 4)
            // (10, 15)
            // <del>I'</del>m ~~super~~
            // (10 + 7, 15 + 7)
            // (17, 22)

            // Solution
            // "<tag></tag>".len() - identifier.len() * 2

            for (s, e) in res {
                line = format!(
                    "{}<{}>{}</{}>{}",
                    &line[..s + offset - pattern.len()],
                    tag_name,
                    &line[s + offset..e + offset],
                    tag_name,
                    &line[e + offset + pattern.len()..]
                );

                // "<>".len() + "del".len() * 2 + "</>".len() - "`".len() * 2
                offset += 2 + (tag_name.len() * 2) + 3 - pattern.len() * 2;
            }

            line
        }

        for (pattern, tag_name) in emph {
            let res = self.capture_simple_pattern(&formatted_line, pattern);

            formatted_line = format(formatted_line, res, pattern, tag_name);
        }

        formatted_line
    }

    fn capture_pattern(
        &self,
        line: &str,
        start_pattern: &str,
        end_pattern: &str,
    ) -> Vec<(usize, usize)> {
        let mut captured = vec![];

        let chars = line.chars();

        // Chain an empty character so that the last iteration can run to
        // catch matches at the end of a line.
        let chars = chars.chain(['‎'].into_iter());

        let mut start_pos = None;
        let mut end_pos = None;
        let mut matches = 0;

        let start_pattern = start_pattern.as_bytes();
        let end_pattern = end_pattern.as_bytes();

        for (idx, e) in chars.enumerate() {
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

            if e == current_pattern[matches] as char {
                matches += 1;
            } else {
                matches = 0;
            }

            if start_pos.is_some() && end_pos.is_some() {
                if end_pos.unwrap() - start_pos.unwrap() >= 1 {
                    captured.push((start_pos.unwrap(), end_pos.unwrap()));
                }
                start_pos = None;
                end_pos = None;
            }
        }

        captured
    }

    fn capture_simple_pattern(&self, line: &str, pattern: &str) -> Vec<(usize, usize)> {
        self.capture_pattern(line, pattern, pattern)
    }

    fn create_tag(&self, tag: &str, content: &str) -> String {
        if content.is_empty() {
            String::new()
        } else {
            format!("<{}>{}</{}>", tag, self.emphasis(content.trim()), tag)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blackquote() {
        let parser = Parser::new();
        let mut result = String::new();

        let blackquote = parser.parse_blockquote("> Yassin Said", &mut result);
        result.push_str(&blackquote);
        let blackquote = parser.parse_blockquote(">", &mut result);
        result.push_str(&blackquote);
        let blackquote = parser.parse_blockquote("> That he's so dumb", &mut result);
        result.push_str(&blackquote);

        assert_eq!(
            result,
            "<blockquote><p>Yassin Said</p> <p>That he's so dumb</p></blockquote>"
        );
    }

    #[test]
    fn header() {
        let mut result = String::new();
        let parser = Parser::new();

        let header = parser.parse_header("# Hey", &mut result);
        assert_eq!(header, "<h1>Hey</h1>");

        let header = parser.parse_header("##### Hola!", &mut result);
        assert_eq!(header, "<h5>Hola!</h5>");
    }

    #[test]
    fn paragraph() {
        let mut result = String::new();
        let parser = Parser::new();

        let paragraph = parser.parse_paragraph("  Hello World  ", &mut result);
        result.push_str(&paragraph);

        assert_eq!(paragraph, "<p>Hello World<br></p>");

        let paragraph = parser.parse_paragraph("I'm Yassin", &mut result);

        result.push_str(&paragraph);
        assert_eq!(paragraph, " I'm Yassin</p>");

        assert_eq!(result, "<p>Hello World<br> I'm Yassin</p>");

        let paragraph = parser.parse_paragraph("", &mut result);
        result.push_str(&paragraph);

        let paragraph = parser.parse_paragraph("#Hello World", &mut result);
        result.push_str(&paragraph);

        assert_eq!(
            result,
            "<p>Hello World<br> I'm Yassin</p> <p>#Hello World</p>"
        );
    }

    #[test]
    fn emphasis() {
        let parser = Parser::new();

        let emphasised = parser.emphasis("**H**ello, I'm *Yassin* not ~~Husien~~. I'm a `coder`");
        assert_eq!(
            emphasised,
            "<b>H</b>ello, I'm <em>Yassin</em> not <del>Husien</del>. I'm a <code>coder</code>"
        );

        let emphasised = parser.emphasis("Can emph las**t**");
        assert_eq!(emphasised, "Can emph las<b>t</b>");

        let emphasised = parser.emphasis("Can emph ***~~nested~~***");
        assert_eq!(emphasised, "Can emph <b><em><del>nested</del></b></em>");

        let emphasised = parser.emphasis("*C*an emph first");
        assert_eq!(emphasised, "<em>C</em>an emph first");
    }

    #[test]
    fn capture_comlex_pattern() {
        let parser = Parser::new();

        let captured = parser.capture_pattern("![link]", "![", "]");
        assert_eq!(captured, vec![(2, 6)]);

        let captured = parser.capture_pattern("*&<link>~!", "*&<", ">~!");
        assert_eq!(captured, vec![(3, 7)]);

        let captured = parser.capture_pattern("^(special) something ^(or) no^(t)", "^(", ")");
        assert_eq!(captured, vec![(2, 9), (23, 25), (31, 32)]);
    }

    #[test]
    fn capture_simple_pattern() {
        let parser = Parser::new();

        let captured = parser.capture_simple_pattern("*&This is a simple line*& *&l*&", "*&");
        assert_eq!(captured, vec![(2, 23), (28, 29)]);

        let captured = parser.capture_simple_pattern("**Th**is **is** a si**mple line**", "**");
        assert_eq!(captured, vec![(2, 4), (11, 13), (22, 31)]);

        let captured = parser.capture_simple_pattern("Last lette`r`", "`");
        assert_eq!(captured, vec![(11, 12)]);

        let captured = parser.capture_simple_pattern("`F`irst letter", "`");
        assert_eq!(captured, vec![(1, 2)]);
    }

    #[test]
    fn create_tag() {
        let parser = Parser::new();

        let tag = parser.create_tag("code", "console.log(\"Hello\")");

        assert_eq!(tag, String::from("<code>console.log(\"Hello\")</code>"));
    }
}
