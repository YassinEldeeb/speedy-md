pub struct Parser {}

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

        // TODO: multiple pattern matches on the same line causes a conflict in the matching range
        // EX: He**l**lo Gu**y**s
        // when inserting <b> like this <b>l</b>
        // the range index of the second match isn't valid
        fn format(
            mut line: String,
            res: Vec<(usize, usize)>,
            pattern: &str,
            tag_name: &str,
        ) -> String {
            for (s, e) in res {
                line = format!(
                    "{}<{}>{}</{}>{}",
                    &line[..s - pattern.len()],
                    tag_name,
                    &line[s..e],
                    tag_name,
                    &line[e + pattern.len()..]
                );
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
                    end_pos = Some(idx - start_pattern.len());
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
                if start_pos.unwrap() - end_pos.unwrap() > 1 {
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
