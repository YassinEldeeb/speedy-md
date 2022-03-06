use std::time::Instant;

#[derive(PartialEq)]
enum Type {
    Header(usize),
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
        Parser {
            position: 0,
            bytes_result: Some(String::new()),
        }
    }

    pub fn get_html(&mut self, content: &str) -> String {
        let bytes = content.as_bytes();

        let headers = ["<h1>", "<h2>", "<h3>", "<h4>", "<h5>", "<h6>"];
        let closed_headers = ["</h1>", "</h2>", "</h3>", "</h4>", "</h5>", "</h6>"];

        let not_end_of_line = |curr_byte| curr_byte != b'\r' && curr_byte != b'\n';

        while self.position < bytes.len() {
            match self.identify_curr_char(bytes) {
                Type::Header(mut size) => {
                    // Subtracting 1 Cause we're indexing, so we start from 0
                    size -= 1;

                    // Push opening tag
                    self.push_str(headers[size]);

                    // Consume spaces after the "#"
                    self.consume_while(bytes, |next_byte| next_byte == b' ');

                    // Consume characters after the "# "
                    let (start, end) = self.consume_while(bytes, not_end_of_line);
                    self.push_str(&content[start..end]);

                    // Push closing tag
                    self.push_str(closed_headers[size]);
                }
                Type::Paragraph => {
                    self.push_str("<p>");

                    let (start, end) = self.consume_while(bytes, not_end_of_line);
                    self.push_str(&content[start..end]);

                    self.push_str("</p>");
                }
                Type::LineBreak => {}
                Type::Space => {}
            };

            self.position += 1;
        }

        let html = self.bytes_result.take().unwrap();

        // Reset values
        self.bytes_result = Some(String::new());
        self.position = 0;

        html
    }

    fn identify_curr_char(&mut self, bytes: &[u8]) -> Type {
        let curr_byte = bytes[self.position];

        if curr_byte == b'#' {
            // Count sequential "#" to determine header size
            let (start, end) = self
                .consume_while(bytes, |next_byte| next_byte == b'#');
            let size = end - start;

            if size > 6 {
                // Go back by "#######".len() to capture those in the <p> content
                self.position -= size;
                Type::Paragraph
            } else {
                Type::Header(size)
            }
        } else if curr_byte == b' ' {
            Type::Space
        } else if curr_byte == b'\r' || curr_byte == b'\n' {
            Type::LineBreak
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
                end +=1 ;
            } else {
                break;
            }

            self.position += 1;
        }

        (start, end)
    }

    fn push_str(&mut self, string: &str) {
        self.bytes_result.as_mut().unwrap().push_str(string);
    }
}
