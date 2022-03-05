#[derive(PartialEq)]
enum Type {
    Header(usize),
    Paragraph,
    LineBreak,
    Space,
}

pub struct Parser {
    position: usize,
    bytes_result: Option<Vec<u8>>,
}

impl Parser {
    pub fn new() -> Self {
        Parser {
            position: 0,
            bytes_result: Some(Vec::new()),
        }
    }

    pub fn get_html(&mut self, content: &str) -> String {
        let bytes = content.as_bytes();

        let headers = [b"<h1>", b"<h2>", b"<h3>", b"<h4>", b"<h5>", b"<h6>"];
        let closed_headers = [b"</h1>", b"</h2>", b"</h3>", b"</h4>", b"</h5>", b"</h6>"];

        let not_end_of_line = |curr_byte| curr_byte != b'\r' && curr_byte != b'\n';

        loop {
            if self.position >= bytes.len() {
                break;
            }

            let curr_byte = bytes[self.position];

            match self.identify_curr_char(bytes) {
                Type::Header(mut size) => {
                    // Subtracting 1 Cause we're indexing, so we start from 0
                    size -= 1;

                    // Push opening tag
                    self.push_bytes(headers[size]);

                    // Consume spaces after the "#"
                    self.consume_while(bytes, |next_byte| next_byte == b' ');

                    // Consume characters after the "# "
                    let header_content = self.consume_while(bytes, not_end_of_line);
                    self.push_bytes(&header_content);

                    // Push closing tag
                    self.push_bytes(closed_headers[size]);
                }
                Type::Paragraph => {
                    self.push_bytes(b"<p>");

                    let paragraph_content = self.consume_while(bytes, not_end_of_line);

                    self.push_bytes(&paragraph_content);
                    self.push_bytes(b"</p>");
                }
                Type::LineBreak => {}
                Type::Space => {}
            };

            self.position += 1;
        }

        String::from_utf8(self.bytes_result.take().unwrap()).expect("Invalid UTF-8 Characters")
    }

    fn identify_curr_char(&mut self, bytes: &[u8]) -> Type {
        let curr_byte = bytes[self.position];

        if curr_byte == b'#' {
            // Count sequential "#" to determine header size
            let size = self
                .consume_while(bytes, |next_byte| next_byte == b'#')
                .len();

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

    fn consume_while<T>(&mut self, bytes: &[u8], condition: T) -> Vec<u8>
    where
        T: Fn(u8) -> bool,
    {
        let mut result = vec![];

        loop {
            if self.position >= bytes.len() {
                break;
            }

            let next_byte = bytes[self.position];

            // Exit the loop if we faced the end of a line
            if condition(next_byte) {
                result.push(next_byte);
            } else {
                break;
            }

            self.position += 1;
        }

        result
    }

    fn push_bytes(&mut self, bytes: &[u8]) {
        for &i in bytes {
            self.bytes_result.as_mut().unwrap().push(i);
        }
    }
}
