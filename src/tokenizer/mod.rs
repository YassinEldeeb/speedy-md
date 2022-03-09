mod utils;
use utils::*;

use std::vec;

#[derive(Debug)]
enum Type<'a> {
    Header(usize),
    Paragraph,
    Blockquote,
    OrderedList,
    UnorderedList,
    Li,
    Link { label: &'a str, url: &'a str },
    LineBreak,
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
    Bold,
    Italic,
    Code,
    Paragraph,
    ClosedParagraph,
    Blockquote,
    ClosedBlockquote,
    OrderedList,
    ClosedOrderedList,
    UnorderedList,
    ClosedUnorderedList,
    Li,
    ClosedLi,
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
        let byte_type = self.identify_byte();

        match byte_type {
            Type::Header(size) => Some(self.tokenize_header(size)),
            Type::Paragraph => Some(self.tokenize_paragraph()),
            Type::Blockquote => Some(self.tokenize_blockquote()),
            Type::UnorderedList => Some(self.tokenize_unordered_list()),
            Type::OrderedList => Some(self.tokenize_ordered_list()),
            Type::Link { label, url } => Some(self.tokenize_link(label, url)),
            Type::HorizontalRule => Some(vec![Token::LineSeparator]),
            _ => None,
        }
    }

    fn check_if_all_next_char_matching(&mut self, ident: char) -> bool {
        let res = self.consume_while_return_str(not_new_line);

        let cond = res.chars().all(|a| a == ident) && res.len() >= 2;

        if cond {
            cond
        } else {
            self.go_back(res.len());
            cond
        }
    }

    /// Identifies the current byte and return a `Token` type
    /// # Example
    /// ```
    /// let token = self.identify_byte();
    /// ```
    fn identify_byte(&mut self) -> Type<'a> {
        let curr_byte = self.next_byte();

        if !not_new_line(curr_byte) {
            return Type::UnRecognized;
        }

        if curr_byte == b'#' {
            let mut size = 1;

            while self.next_byte() == b'#' {
                size += 1;
            }

            // Go back to get the last not matching character
            self.go_back(1);

            if size > 6 || self.seek_next_byte() != b' ' {
                self.go_back(size);
                Type::Paragraph
            } else {
                Type::Header(size)
            }
        } else if curr_byte == b'>' {
            // Don't Go back cause we want to escape the ">"
            Type::Blockquote
        } else if curr_byte == b'-' && self.seek_next_byte() == b' ' {
            Type::UnorderedList
        } else if (curr_byte as char).is_numeric()
            && is_string_numeric(
                self.consume_while_return_str_without_inc(not_new_line)
                    .split_once(".")
                    .unwrap()
                    .0,
            )
        {
            Type::OrderedList
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

    fn tokenize_link(&mut self, label: &'a str, url: &'a str) -> Vec<Token<'a>> {
        vec![Token::Link { label, url }]
    }

    fn tokenize_ordered_list(&mut self) -> Vec<Token<'a>> {
        vec![
            Token::OrderedList,
            Token::Li,
            Token::Text(
                self.consume_while_return_str(not_new_line)
                    .split_once(".")
                    .unwrap()
                    .1
                    .trim_start(),
            ),
            Token::ClosedLi,
            Token::ClosedOrderedList,
        ]
    }

    fn tokenize_unordered_list(&mut self) -> Vec<Token<'a>> {
        self.consume_whitespace();

        vec![
            Token::UnorderedList,
            Token::Li,
            Token::Text(self.consume_while_return_str(not_new_line)),
            Token::ClosedLi,
            Token::ClosedUnorderedList,
        ]
    }

    fn tokenize_blockquote(&mut self) -> Vec<Token<'a>> {
        let mut result = vec![Token::Blockquote];

        self.consume_whitespace();

        let tokens = self.tokenize();

        if let Some(tokens) = tokens {
            for t in tokens {
                result.push(t)
            }
        }

        result.push(Token::ClosedBlockquote);

        result
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
        let mut result = vec![Token::Paragraph];
        let text = self.consume_while_return_str(not_new_line);

        if text.ends_with("  ") {
            result.push(Token::Text(text.trim_end()));
            result.push(Token::LineBreak);
        } else {
            result.push(Token::Text(text))
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

    /// Consumes all bytes
    /// # Example
    /// ```
    /// self.consume_while(|byte| byte == b' ');
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

    /// Consumes all bytes
    /// # Example
    /// ```
    /// let content = self.consume_while_return_str(|byte| byte == b' ');
    /// ```
    fn consume_while_return_str_without_inc<F>(&mut self, condition: F) -> &'a str
    where
        F: Fn(u8) -> bool,
    {
        let (s, e) = self.consume_while_without_inc(condition);

        &self.content[s..e]
    }

    /// Consumes all bytes
    /// # Example
    /// ```
    /// let content = self.consume_while_return_str(|byte| byte == b' ');
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
