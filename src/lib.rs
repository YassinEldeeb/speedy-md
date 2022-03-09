use std::vec;

enum Type {
    Header(usize),
    Paragraph,
    UnRecognized,
}

#[derive(Debug)]
pub enum Token<'a> {
    Header(usize),
    ClosedHeader(usize),
    Text(&'a str),
    Bold,
    Italic,
    Code,
    Paragraph,
    ClosedParagraph,
}

#[derive(Debug)]
pub struct Tokenizer<'a> {
    position: usize,
    bytes: &'a [u8],
    content: &'a str,
    result: Option<Vec<Token<'a>>>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(content: &'a str) -> Self {
        Tokenizer {
            content,
            position: 0,
            bytes: content.as_bytes(),
            result: Some(Vec::new()),
        }
    }

    pub fn run(&mut self) -> Vec<Token<'a>> {
        while self.position < self.bytes.len() {
            // Consume white spaces before identifying the line
            self.consume_whitespace();

            let tokens = self.tokenize();

            if let Some(tokens) = tokens {
                for t in tokens {
                    self.result.as_mut().unwrap().push(t)
                }
            }
        }

        self.result.take().unwrap()
    }

    fn tokenize(&mut self) -> Option<Vec<Token<'a>>> {
        match self.identify_byte() {
            Type::Header(size) => Some(self.tokenize_header(size)),
            Type::Paragraph => Some(self.tokenize_paragraph()),
            _ => None,
        }
    }

    /// Identifies the current byte and return a `Token` type
    /// # Example
    /// ```
    /// let token = self.identify_byte();
    /// ```
    fn identify_byte(&mut self) -> Type {
        let curr_byte = self.next_byte();

        if !not_new_line(curr_byte) {
            return Type::UnRecognized;
        }

        if curr_byte == b'#' {
            let mut size = 1;

            while self.seek_next_byte() == b'#' {
                size += 1;

                if size >= 7 {
                    break;
                }
            }

            // Go forward `size` - 1 to skip the "###"
            self.go_forward(size - 1);

            Type::Header(size)
        } else {
            // Go back 1 to get the first char we checked
            self.go_back(1);

            Type::Paragraph
        }
    }

    fn tokenize_header(&mut self, size: usize) -> Vec<Token<'a>> {
        if size >= 7 || self.next_byte() != b' ' {
            self.go_back(size);

            self.tokenize_paragraph()
        } else {
            self.consume_whitespace();

            vec![
                Token::Header(size),
                Token::Text(self.consume_while_return_str(not_new_line)),
                Token::ClosedHeader(size),
            ]
        }
    }

    fn tokenize_paragraph(&mut self) -> Vec<Token<'a>> {
        vec![
            Token::Paragraph,
            Token::Text(self.consume_while_return_str(not_new_line)),
            Token::ClosedParagraph,
        ]
    }

    /// Consumes all leading whitespaces
    /// # Example
    /// ```
    /// self.consume_whitespace();
    /// ```
    fn consume_whitespace(&mut self) {
        self.consume_while(|byte| byte == b' ');
    }

    /// Consumes all bytes
    /// # Example
    /// ```
    /// self.consume_whitespace();
    /// ```
    fn consume_while<F>(&mut self, condition: F) -> (usize, usize)
    where
        F: Fn(u8) -> bool,
    {
        let start = self.position;

        while self.position < self.bytes.len() {
            if !condition(self.next_byte()) {
                self.go_back(1);
                break;
            }
        }

        (start, self.position)
    }

    /// Consumes all bytes
    /// # Example
    /// ```
    /// self.consume_whitespace();
    /// ```
    fn consume_while_return_str<F>(&mut self, condition: F) -> &'a str
    where
        F: Fn(u8) -> bool,
    {
        let (s, e) = self.consume_while(condition);

        &self.content[s..e]
    }

    /// Get the next byte while also incrementing the position of the tokenizer
    /// # Example
    /// ```
    /// let next_byte = self.next_byte();
    /// ```
    fn next_byte(&mut self) -> u8 {
        let byte = self.bytes[self.position];
        self.go_forward(1);

        byte
    }

    fn go_back(&mut self, num: usize) {
        self.position -= num;
    }

    fn go_forward(&mut self, num: usize) {
        self.position += num;
    }

    /// Get the next byte without changing the position of the tokenizer
    /// # Example
    /// ```
    /// let next_byte = self.seek_next_byte();
    /// ```
    fn seek_next_byte(&mut self) -> u8 {
        let byte = self.bytes[self.position];

        byte
    }

    /// Get the last byte without changing the position of the tokenizer
    /// # Example
    /// ```
    /// let last_byte = self.last_byte();
    /// ```
    fn last_byte(&mut self) -> u8 {
        let byte = self.bytes[self.position - 1];

        byte
    }
}

fn not_new_line(b: u8) -> bool {
    b != b'\n' && b != b'\r'
}
