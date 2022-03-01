pub struct Parser {}

// Remember to check this out when implementing merging <p> tags
// And catching blockquotes
// https://www.google.com/search?q=how+to+skip+the+next+3+iterations+of+a+loop+in+rust&client=firefox-b-d&channel=trow5&sxsrf=APq-WBuGqOp0npRLcHpjdEfkOjmCYm8Z5g%3A1646140375716&ei=1xseYt-eK5P8sAeLnLWIAQ&ved=0ahUKEwif5uDi_qT2AhUTPuwKHQtODREQ4dUDCA0&uact=5&oq=how+to+skip+the+next+3+iterations+of+a+loop+in+rust&gs_lcp=Cgdnd3Mtd2l6EAMyBAghEBU6BwgAEEcQsANKBAhBGABKBAhGGABQvgdYvQpgmQtoA3ABeACAAbcBiAH3A5IBAzAuM5gBAKABAcgBCMABAQ&sclient=gws-wiz

impl Parser {
    pub fn new() -> Self {
        Parser {}
    }

    pub fn get_html(&self, content: String) -> String {
        let mut result = String::new();

        let lines = content.lines().map(|l| l.trim());
        for line in lines {
            if line.starts_with("#") {
                result.push_str(&self.identify_header(line));
            } else if line.is_empty() {
            } else {
                result.push_str(&self.identify_paragraph(line));
            }
        }

        result
    }

    fn identify_header(&self, line: &str) -> String {
        let mut size = 0;

        let mut chars = line.chars();
        while chars.next() == Some('#') {
            size += 1;
        }

        // TODO: If size is more than 6 then it's a <p>

        let line = &line[size + 1..];
        self.create_tag(&format!("h{}", size), line)
    }

    fn identify_paragraph(&self, line: &str) -> String {
        // TODO: Merge lines in a single <p> if there was no `\n` between them
        self.create_tag(&format!("p"), line)
    }

    fn emphasis(&self, line: &str) -> String {
        let emph = vec![("**", "b"), ("*", "em"), ("_", "em"), ("~~", "del")];

        let mut formatted_line = String::from(line);

        // DONE: multiple pattern matches on the same line causes a conflict in the matching range
        // EX: He**l**lo Gu**y**s
        // when inserting <b> like this <b>l</b>
        // the range index of the second match isn't valid
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
            // "<tag></tag>".len() - identifier.len()
            for (s, e) in res {
                line = format!(
                    "{}<{}>{}</{}>{}",
                    &line[..s + offset - pattern.len()],
                    tag_name,
                    &line[s + offset..e + offset],
                    tag_name,
                    &line[e + offset + pattern.len()..]
                );

                offset += 2 + tag_name.len() * 2 + 3 - pattern.len();
            }

            line
        }

        for (pattern, tag_name) in emph {
            let res = self.capture_simple_pattern(&formatted_line, pattern);
            formatted_line = format(formatted_line, res, pattern, tag_name);
        }

        formatted_line
    }

    // Pattern
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
                if end_pos.unwrap() - start_pos.unwrap() > 1 {
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
        format!("<{}>{}</{}>", tag, self.emphasis(content), tag)
    }
}
