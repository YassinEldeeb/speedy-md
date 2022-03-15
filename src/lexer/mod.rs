mod utils;
use std::vec;
use utils::*;

#[derive(Debug, PartialEq, Clone)]
enum Type<'a> {
    Header(usize),
    Paragraph,
    Blockquote,
    OrderedList(u32),
    UnorderedList,
    Link { label: &'a str, url: &'a str },
    HorizontalRule,
    UnRecognized,
}

#[derive(Debug)]
pub enum Token<'a> {
    Header(usize),
    ClosedHeader(usize),
    Text(&'a str),
    Link { label: &'a str, url: &'a str },
    LineBreak,
    LineSeparator,
    Paragraph,
    ClosedParagraph,
    Blockquote,
    ClosedBlockquote,
    OrderedList(u32),
    ClosedOrderedList,
    UnorderedList,
    ClosedUnorderedList,
    Li,
    ClosedLi,
    // Emphasis
    Bold,
    ClosedBold,
    Italic,
    ClosedItalic,
    Code,
    ClosedCode,
}

#[derive(Debug)]
pub struct Lexer<'a> {
    position: usize,
    bytes: &'a [u8],
    content: &'a str,
    result: Option<Vec<Token<'a>>>,
}

impl<'a> Lexer<'a> {
    pub fn new(content: &'a str) -> Self {
        Lexer {
            content,
            position: 0,
            bytes: content.as_bytes(),
            result: Some(Vec::new()),
        }
    }

    pub fn run(&mut self) -> Vec<Token<'a>> {
        while !self.end_of_content() {
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
        if self.end_of_content() {
            return None;
        }

        let byte_type = self.identify_byte();

        match byte_type {
            Type::Header(size) => Some(self.tokenize_header(size)),
            Type::Paragraph => Some(self.tokenize_paragraph()),
            // Type::Blockquote => Some(self.tokenize_blockquote()),
            // Type::UnorderedList => Some(self.tokenize_unordered_list()),
            // Type::OrderedList(start) => Some(self.tokenize_ordered_list(start)),
            // Type::Link { label, url } => Some(self.tokenize_link(label, url)),
            Type::HorizontalRule => Some(vec![Token::LineSeparator]),
            _ => None,
        }
    }

    fn check_if_all_next_char_matching(&mut self, ident: char) -> bool {
        let curr_pos = self.position;
        let res = self.consume_while_return_str(|b| not_new_line(b));

        let cond = res.chars().all(|a| a == ident) && res.len() >= 2;

        if cond {
            cond
        } else {
            self.go_back(self.position - curr_pos);
            cond
        }
    }

    /// Identifies the current byte and return a `Token` type
    /// # Example
    /// ```
    /// let token = self.identify_byte();
    /// ```
    fn identify_byte(&mut self) -> Type<'a> {
        let curr_byte = self.next_byte().unwrap();

        if is_new_line(curr_byte) {
            return Type::UnRecognized;
        }

        if curr_byte == b'#' {
            let mut size = 1;

            while !self.end_of_content() && self.next_byte().unwrap() == b'#' {
                size += 1;
            }

            // Go back to get the last not matching character
            self.go_back(1);

            if size > 6 || self.seek_next_byte().unwrap() != b' ' {
                self.go_back(size);
                Type::Paragraph
            } else {
                Type::Header(size)
            }
        } else if curr_byte == b'>' {
            // Don't Go back cause we want to escape the ">"
            Type::Blockquote
        } else if curr_byte == b'-' && self.seek_next_byte().unwrap() == b' ' {
            Type::UnorderedList
        } else if (curr_byte as char).is_numeric()
            && is_string_numeric(
                self.consume_while_return_str_without_inc(not_new_line)
                    .split_once(".")
                    .unwrap()
                    .0,
            )
        {
            self.go_back(1);

            let num = self
                .consume_while_return_str_without_inc(not_new_line)
                .split_once(".")
                .unwrap()
                .0;

            // Skip "12".len() + ".".len() + next_char that we return back previously
            self.go_forward(num.len() + 2);

            Type::OrderedList(num.parse().unwrap())
        } else if (curr_byte == b'*' || curr_byte == b'-' || curr_byte == b'_')
            && (self.check_if_all_next_char_matching('*')
                || self.check_if_all_next_char_matching('-')
                || self.check_if_all_next_char_matching('_'))
        {
            Type::HorizontalRule
        } else {
            // Go back to get the last not matching character
            self.go_back(1);

            Type::Paragraph
        }
    }

    fn tokenize_header(&mut self, size: usize) -> Vec<Token<'a>> {
        if size >= 7 || self.next_byte().unwrap() != b' ' {
            self.go_back(size);

            self.tokenize_paragraph()
        } else {
            self.consume_whitespace();
            let mut result = vec![Token::Header(size)];

            let text = self.consume_while_return_str(not_new_line);
            result.push(Token::Text(text));
            result.push(Token::ClosedHeader(size));

            result
        }
    }

    fn tokenize_paragraph(&mut self) -> Vec<Token<'a>> {
        let mut result = vec![Token::Paragraph];
        let text = self.consume_paragraph();

        let lines: Vec<&str> = text.lines().collect();
        for (idx, &l) in lines.iter().enumerate() {
            let line_break = l.ends_with("  ");

            result.push(Token::Text(l.trim_end()));

            if line_break {
                result.push(Token::LineBreak)
            } else if idx < lines.len() - 1 {
                result.push(Token::Text(" "));
            }
        }

        result.push(Token::ClosedParagraph);

        result
    }

    /// Consumes all leading whitespaces
    /// # Example
    /// ```
    /// self.consume_whitespace();
    /// ```
    fn consume_whitespace(&mut self) {
        self.consume_while(|byte| byte == b' ');
    }

    /// Consumes all bytes and return the `(start, end)` positions based
    /// on the condition you've provided
    /// # Example
    /// ```
    /// self.consume_paragraph();
    /// ```
    fn consume_paragraph(&mut self) -> &'a str {
        let start = self.position;

        let mut last_line_type = Type::UnRecognized;
        let mut curr_line_type = Type::UnRecognized;

        while !self.end_of_content() {
            let curr = self.position;
            let next_byte = self.next_byte().unwrap();

            if is_new_line(next_byte) || curr_line_type == Type::UnRecognized {
                let curr_pos = self.position;

                self.go_back(curr_pos - curr);
                let t = self.identify_byte();
                let offset = self.position - curr_pos;

                self.go_back(offset);

                last_line_type = curr_line_type;
                curr_line_type = t;
            }

            let two_seq_line_breaks =
                curr_line_type == Type::UnRecognized && last_line_type == Type::UnRecognized;

            if curr_line_type != Type::Paragraph && curr_line_type != Type::UnRecognized
                || two_seq_line_breaks
            {
                if two_seq_line_breaks {
                    self.go_back(2);
                }
                self.go_back(1);
                break;
            }
        }

        &self.content[start..self.position]
    }

    /// Consumes all bytes and return the `(start, end)` positions based
    /// on the condition you've provided
    /// # Example
    /// ```
    /// self.consume_while(|byte| byte == b' ');
    /// ```
    fn consume_while<F>(&mut self, condition: F) -> (usize, usize)
    where
        F: Fn(u8) -> bool,
    {
        let start = self.position;

        while !self.end_of_content() {
            let next_byte = self.next_byte().unwrap();

            if !condition(next_byte) {
                self.go_back(1);
                break;
            }
        }

        (start, self.position)
    }

    /// Check if the currrent position is at the end of the string
    fn end_of_content(&self) -> bool {
        self.position >= self.bytes.len()
    }

    /// Consumes all bytes
    /// # Example
    /// ```
    /// self.consume_while(|byte| byte == b' ');
    /// ```
    fn consume_while_without_inc<F>(&mut self, condition: F) -> (usize, usize)
    where
        F: Fn(u8) -> bool,
    {
        let start = self.position;
        let mut end = self.position;

        while end < self.bytes.len() {
            if !condition(self.bytes[end]) {
                break;
            } else {
                end += 1;
            }
        }

        (start, end)
    }

    /// Consumes all bytes and return the string based on the condition you provided
    /// Without incrementing the Lexer position
    /// # Example
    /// ```
    /// let content = self.consume_while_return_str_without_inc(|byte| byte == b' ');
    /// ```
    fn consume_while_return_str_without_inc<F>(&mut self, condition: F) -> &'a str
    where
        F: Fn(u8) -> bool,
    {
        let (s, e) = self.consume_while_without_inc(condition);

        &self.content[s..e]
    }

    /// Consumes all bytes and return the string based on the condition you provided
    /// # Example
    /// ```
    /// let content = self.consume_while_return_str(|byte| byte == b' ');
    /// ```
    fn consume_while_return_str<F>(&mut self, condition: F) -> &'a str
    where
        F: Fn(u8) -> bool,
    {
        let (s, e) = self.consume_while(condition);

        if self.content[s..e].ends_with('\r') {
            &self.content[s..e - 1]
        } else {
            &self.content[s..e]
        }
    }

    /// Get the next byte while also incrementing the position of the Lexer
    /// # Example
    /// ```
    /// let next_byte = self.next_byte();
    /// ```
    fn next_byte(&mut self) -> Option<u8> {
        if self.end_of_content() {
            None
        } else {
            let byte = self.bytes[self.position];

            if byte == b'\r' {
                let next_byte = self.bytes[self.position + 1];
                self.go_forward(2);
                Some(next_byte)
            } else {
                self.go_forward(1);
                Some(byte)
            }
        }
    }

    fn go_back(&mut self, num: usize) {
        self.position -= num;
    }

    fn go_forward(&mut self, num: usize) {
        self.position += num;
    }

    /// Get the next byte without changing the position of the Lexer
    /// # Example
    /// ```
    /// let next_byte = self.seek_next_byte();
    /// ```
    fn seek_next_byte(&mut self) -> Option<u8> {
        if self.end_of_content() {
            None
        } else {
            let byte = self.bytes[self.position];

            if byte == b'\r' {
                Some(self.bytes[self.position + 1])
            } else {
                Some(byte)
            }
        }
    }

    /// Get the last byte without changing the position of the Lexer
    /// # Example
    /// ```
    /// let last_byte = self.last_byte();
    /// ```
    fn last_byte(&mut self) -> u8 {
        let byte = self.bytes[self.position - 1];

        if byte == b'\r' {
            self.bytes[self.position - 2]
        } else {
            byte
        }
    }
}
