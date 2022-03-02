use std::vec;

pub struct Parser {}

impl Parser {
    pub fn new() -> Self {
        Parser {}
    }

    pub fn get_html(&self, content: String) -> String {
        let mut result = String::new();

        let lines: Vec<&str> = content.lines().collect();

        for (idx, line) in lines.iter().enumerate() {
            if line.starts_with("#") {
                let header = &self.identify_header(line, &mut result);
                result.push_str(header);
            } else if line.starts_with(">") {
                let blockquote = &self.identify_blockquote(line, &mut result);
                result.push_str(blockquote);
            } else {
                let paragraph = &self.identify_paragraph(line, &mut result);

                result.push_str(paragraph);
            }
        }

        result
    }

    fn identify_blockquote(&self, line: &str, result: &mut String) -> String {
        let closed_blockquote_tag = "</blockquote>";
        let line = line[1..].trim_start();

        if result.ends_with(closed_blockquote_tag) {
            result.drain(result.len() - closed_blockquote_tag.len()..);

            format!("{}</blockquote>", self.identify_paragraph(line, result))
        } else {
            self.create_tag("blockquote", &self.identify_paragraph(line, result))
        }
    }

    fn identify_header(&self, line: &str, result: &mut String) -> String {
        let mut size = 0;

        let mut chars = line.chars();
        while chars.next() == Some('#') {
            size += 1;
        }

        // # Hello -> " " 👌
        // #Hello -> "H" 👎
        let space_separator = &line[size..size + 1];
        if size > 6 || space_separator != " " {
            return self.identify_paragraph(line, result);
        }

        let line = &line[size..];
        self.create_tag(&format!("h{}", size), line)
    }

    fn identify_paragraph(&self, line: &str, result: &mut String) -> String {
        let closed_p_tag = "</p>";
        if result.ends_with(closed_p_tag) && !line.is_empty() {
            result.drain(result.len() - closed_p_tag.len()..);

            format!(" {}</p>", line)
        } else {
            self.create_tag(&format!("p"), line)
        }
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
            // Push an empty space so that backward inspection
            // can detect where is a linebreak between lines
            String::from(" ")
        } else {
            let tag_value = if content.ends_with("  ") {
                self.emphasis(content.trim()) + "<br>"
            } else {
                self.emphasis(content.trim())
            };
            format!("<{}>{}</{}>", tag, tag_value, tag)
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

        let blackquote = parser.identify_blockquote("> Yassin Said", &mut result);
        result.push_str(&blackquote);
        let blackquote = parser.identify_blockquote(">", &mut result);
        result.push_str(&blackquote);
        let blackquote = parser.identify_blockquote("> That he's so dumb", &mut result);
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

        let header = parser.identify_header("# Hey", &mut result);
        assert_eq!(header, "<h1>Hey</h1>");

        let header = parser.identify_header("##### Hola!", &mut result);
        assert_eq!(header, "<h5>Hola!</h5>");
    }

    #[test]
    fn paragraph() {
        let mut result = String::new();
        let parser = Parser::new();

        let paragraph = parser.identify_paragraph("  Hello World  ", &mut result);
        result.push_str(&paragraph);

        assert_eq!(paragraph, "<p>Hello World<br></p>");

        let paragraph = parser.identify_paragraph("I'm Yassin", &mut result);

        result.push_str(&paragraph);
        assert_eq!(paragraph, " I'm Yassin</p>");

        assert_eq!(result, "<p>Hello World<br> I'm Yassin</p>");

        let paragraph = parser.identify_paragraph("", &mut result);
        result.push_str(&paragraph);

        let paragraph = parser.identify_paragraph("#Hello World", &mut result);
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
