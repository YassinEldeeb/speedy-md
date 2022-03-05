pub struct Parser {
    position: usize,
}

impl Parser {
    pub fn new() -> Self {
        Parser { position: 0 }
    }

    pub fn get_html(&mut self, content: &str) -> String {
        let mut bytes_result = Vec::new();

        let bytes = content.as_bytes();

        let headers = [b"<h1>", b"<h2>", b"<h3>", b"<h4>", b"<h5>", b"<h6>"];
        let closed_headers = [b"</h1>", b"</h2>", b"</h3>", b"</h4>", b"</h5>", b"</h6>"];
        let is_end_of_line = |curr_byte| curr_byte == b'\r' || curr_byte == b'\n';

        loop {
            if self.position >= bytes.len() {
                break;
            }

            let curr_byte = bytes[self.position];

            if is_end_of_line(curr_byte) {
                self.position += 1;
                continue;
            }

            // Identify Headers
            if curr_byte == b'#' {
                // Count sequential "#" to determine header size
                let size = self
                    .consume_while(bytes, |next_byte| next_byte == b'#')
                    .len();

                // If size more than 6 then it's <p>
                if size > 5 {
                    self.push_bytes(&mut bytes_result, b"<p>");

                    // Go back by "#######".len() to capture those in the <p> content
                    self.position -= size;

                    // Consume characters after the "#######"
                    let paragraph_content = self.consume_while(bytes, is_end_of_line);

                    self.push_bytes(&mut bytes_result, &paragraph_content);
                    self.push_bytes(&mut bytes_result, b"</p>");
                } else {
                    // Push opening tag
                    self.push_bytes(&mut bytes_result, headers[size]);

                    // Consume spaces after the "#"
                    self.consume_while(bytes, |next_byte| next_byte == b' ');

                    // Consume characters after the "# "
                    let header_content = self.consume_while(bytes, is_end_of_line);
                    self.push_bytes(&mut bytes_result, &header_content);

                    // Push closing tag
                    self.push_bytes(&mut bytes_result, closed_headers[size]);
                }
            } else {
                self.push_bytes(&mut bytes_result, b"<p>");
                let paragraph_content = self.consume_while(bytes, is_end_of_line);

                self.push_bytes(&mut bytes_result, &paragraph_content);
                self.push_bytes(&mut bytes_result, b"</p>");
            }

            self.position += 1;
        }

        String::from_utf8(bytes_result).expect("Invalid UTF-8 Characters")
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

    fn push_bytes(&self, bytes_result: &mut Vec<u8>, bytes: &[u8]) {
        for &i in bytes {
            bytes_result.push(i);
        }
    }
}
