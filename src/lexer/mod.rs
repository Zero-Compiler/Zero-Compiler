
pub mod token;
pub mod token_preprocessor;

use token::{Token, TokenType, Position};
pub use token_preprocessor::{TokenPreprocessor, ScientificNotationAnalyzer, InferredNumericType};
pub use crate::error::{CompilerError as LexerError};

pub type LexerResult<T> = Result<T, LexerError>;

/// 词法分析器主结构
pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
    current_char: Option<char>,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        let chars: Vec<char> = input.chars().collect();
        let current_char = chars.get(0).copied();
        Lexer {
            input: chars,
            position: 0,
            line: 1,
            column: 1,
            current_char,
        }
    }

    /// 前进到下一个字符，处理UTF-8和行列追踪
    fn advance(&mut self) {
        if let Some(ch) = self.current_char {
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                // UTF-8字符宽度处理
                self.column += Self::char_display_width(ch);
            }
        }
        
        self.position += 1;
        self.current_char = self.input.get(self.position).copied();
    }

    /// 计算字符的显示宽度（用于正确的列位置计算）
    fn char_display_width(ch: char) -> usize {
        // 简化版本：大多数字符宽度为1，某些CJK字符为2
        if ch.is_ascii() {
            1
        } else {
            // Unicode东亚宽度检测（简化版）
            let code = ch as u32;
            if (0x1100..=0x115F).contains(&code) || 
               (0x2E80..=0x9FFF).contains(&code) ||
               (0xAC00..=0xD7A3).contains(&code) ||
               (0xF900..=0xFAFF).contains(&code) ||
               (0xFF00..=0xFF60).contains(&code) ||
               (0xFFE0..=0xFFE6).contains(&code) {
                2
            } else {
                1
            }
        }
    }

    /// 获取当前位置信息
    fn current_position(&self) -> Position {
        Position::new(self.line, self.column, self.position)
    }

    /// 向前看指定偏移量的字符
    fn peek(&self, offset: usize) -> Option<char> {
        self.input.get(self.position + offset).copied()
    }

    /// 跳过空白字符
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// 跳过单行注释
    fn skip_comment(&mut self) {
        if self.current_char == Some('/') && self.peek(1) == Some('/') {
            while self.current_char.is_some() && self.current_char != Some('\n') {
                self.advance();
            }
            if self.current_char == Some('\n') {
                self.advance();
            }
        }
    }

    /// 读取数字（支持多种进制和科学计数法）
    fn read_number(&mut self) -> LexerResult<Token> {
        let start_pos = self.current_position();
        let mut value = String::new();
        let mut is_float = false;
        let mut has_exponent = false;
        
        // 检查进制前缀
        if self.current_char == Some('0') {
            value.push('0');
            self.advance();
            
            match self.current_char {
                Some('x') | Some('X') => {
                    // 十六进制
                    value.push('x');
                    self.advance();
                    return self.read_hex_number(start_pos, value);
                }
                Some('b') | Some('B') => {
                    // 二进制
                    value.push('b');
                    self.advance();
                    return self.read_binary_number(start_pos, value);
                }
                Some('o') | Some('O') => {
                    // 八进制
                    value.push('o');
                    self.advance();
                    return self.read_octal_number(start_pos, value);
                }
                _ => {}
            }
        }

        // 读取整数部分
        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() || ch == '_' {
                if ch != '_' {
                    value.push(ch);
                }
                self.advance();
            } else {
                break;
            }
        }

        // 检查小数点
        if self.current_char == Some('.') && self.peek(1).map_or(false, |c| c.is_ascii_digit()) {
            is_float = true;
            value.push('.');
            self.advance();
            
            while let Some(ch) = self.current_char {
                if ch.is_ascii_digit() || ch == '_' {
                    if ch != '_' {
                        value.push(ch);
                    }
                    self.advance();
                } else {
                    break;
                }
            }
        }

        // 检查科学计数法
        if let Some('e') | Some('E') = self.current_char {
            has_exponent = true;
            value.push('e');
            self.advance();
            
            // 可选的正负号
            if let Some('+') | Some('-') = self.current_char {
                value.push(self.current_char.unwrap());
                self.advance();
            }
            
            // 指数部分
            let exp_start = value.len();
            while let Some(ch) = self.current_char {
                if ch.is_ascii_digit() || ch == '_' {
                    if ch != '_' {
                        value.push(ch);
                    }
                    self.advance();
                } else {
                    break;
                }
            }
            
            if value.len() == exp_start {
                return Err(LexerError::invalid_number(value.clone(), start_pos.line, start_pos.column, start_pos.offset));
            }
        }

        let end_pos = self.current_position();
        
        // 确定token类型
        let token_type = if has_exponent {
            TokenType::ScientificExponent
        } else if is_float {
            TokenType::Float
        } else {
            TokenType::Integer
        };

        Ok(Token::new(token_type, value, start_pos, end_pos))
    }

    /// 读取十六进制数
    fn read_hex_number(&mut self, start_pos: Position, mut value: String) -> LexerResult<Token> {
        while let Some(ch) = self.current_char {
            if ch.is_ascii_hexdigit() || ch == '_' {
                if ch != '_' {
                    value.push(ch);
                }
                self.advance();
            } else {
                break;
            }
        }
        
        if value.len() <= 2 {
            return Err(LexerError::invalid_number(value, start_pos.line, start_pos.column, start_pos.offset));
        }
        
        let end_pos = self.current_position();
        Ok(Token::new(TokenType::Integer, value, start_pos, end_pos))
    }

    /// 读取二进制数
    fn read_binary_number(&mut self, start_pos: Position, mut value: String) -> LexerResult<Token> {
        while let Some(ch) = self.current_char {
            if ch == '0' || ch == '1' || ch == '_' {
                if ch != '_' {
                    value.push(ch);
                }
                self.advance();
            } else {
                break;
            }
        }
        
        if value.len() <= 2 {
            return Err(LexerError::invalid_number(value, start_pos.line, start_pos.column, start_pos.offset));
        }
        
        let end_pos = self.current_position();
        Ok(Token::new(TokenType::Integer, value, start_pos, end_pos))
    }

    /// 读取八进制数
    fn read_octal_number(&mut self, start_pos: Position, mut value: String) -> LexerResult<Token> {
        while let Some(ch) = self.current_char {
            if ('0'..='7').contains(&ch) || ch == '_' {
                if ch != '_' {
                    value.push(ch);
                }
                self.advance();
            } else {
                break;
            }
        }
        
        if value.len() <= 2 {
            return Err(LexerError::invalid_number(value, start_pos.line, start_pos.column, start_pos.offset));
        }
        
        let end_pos = self.current_position();
        Ok(Token::new(TokenType::Integer, value, start_pos, end_pos))
    }

    /// 读取标识符（支持UTF-8）
    fn read_identifier(&mut self) -> Token {
        let start_pos = self.current_position();
        let mut value = String::new();

        while let Some(ch) = self.current_char {
            if ch.is_alphanumeric() || ch == '_' || (!ch.is_ascii() && ch.is_alphabetic()) {
                value.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        let end_pos = self.current_position();
        let token_type = TokenType::get_keyword(&value).unwrap_or(TokenType::Identifier);
        
        Token::new(token_type, value, start_pos, end_pos)
    }

    /// 读取字符串（支持转义序列和Unicode）
    fn read_string(&mut self) -> LexerResult<Token> {
        let start_pos = self.current_position();
        self.advance(); // 跳过开始引号
        
        let mut value = String::new();

        while let Some(ch) = self.current_char {
            if ch == '"' {
                break;
            }
            
            if ch == '\\' {
                self.advance();
                value.push_str(&self.read_escape_sequence()?);
            } else if ch == '\n' {
                // 支持多行字符串
                value.push(ch);
                self.advance();
            } else {
                value.push(ch);
                self.advance();
            }
        }

        if self.current_char != Some('"') {
            return Err(LexerError::unterminated_string(start_pos.line, start_pos.column, start_pos.offset));
        }

        self.advance(); // 跳过结束引号
        let end_pos = self.current_position();

        Ok(Token::new(TokenType::String, value, start_pos, end_pos))
    }

    /// 读取Raw字符串（不处理转义）
    fn read_raw_string(&mut self) -> LexerResult<Token> {
        let start_pos = self.current_position();
        self.advance(); // 跳过 'r'
        
        if self.current_char != Some('"') {
            return Err(LexerError::invalid_character(self.current_char.unwrap_or('\0'), self.line, self.column, self.position));
        }
        
        self.advance(); // 跳过开始引号
        let mut value = String::new();

        while let Some(ch) = self.current_char {
            if ch == '"' {
                break;
            }
            value.push(ch);
            self.advance();
        }

        if self.current_char != Some('"') {
            return Err(LexerError::unterminated_string(start_pos.line, start_pos.column, start_pos.offset));
        }

        self.advance(); // 跳过结束引号
        let end_pos = self.current_position();

        Ok(Token::new(TokenType::String, value, start_pos, end_pos))
    }

    /// 读取字符字面量
    fn read_char(&mut self) -> LexerResult<Token> {
        let start_pos = self.current_position();
        self.advance(); // 跳过开始单引号
        
        let mut value = String::new();

        if let Some(ch) = self.current_char {
            if ch == '\\' {
                self.advance();
                value = self.read_escape_sequence()?;
            } else if ch != '\'' {
                value.push(ch);
                self.advance();
            }
        }

        if self.current_char != Some('\'') {
            return Err(LexerError::unterminated_string(start_pos.line, start_pos.column, start_pos.offset));
        }

        self.advance(); // 跳过结束单引号
        let end_pos = self.current_position();

        Ok(Token::new(TokenType::Char, value, start_pos, end_pos))
    }

    /// 读取转义序列
    fn read_escape_sequence(&mut self) -> LexerResult<String> {
        let line = self.line;
        let column = self.column;
        
        match self.current_char {
            Some('n') => {
                self.advance();
                Ok("\n".to_string())
            }
            Some('t') => {
                self.advance();
                Ok("\t".to_string())
            }
            Some('r') => {
                self.advance();
                Ok("\r".to_string())
            }
            Some('\\') => {
                self.advance();
                Ok("\\".to_string())
            }
            Some('"') => {
                self.advance();
                Ok("\"".to_string())
            }
            Some('\'') => {
                self.advance();
                Ok("'".to_string())
            }
            Some('0') => {
                self.advance();
                Ok("\0".to_string())
            }
            Some('x') => {
                // 十六进制转义 \xHH
                self.advance();
                self.read_hex_escape(line, column)
            }
            Some('u') => {
                // Unicode转义
                self.advance();
                self.read_unicode_escape(line, column)
            }
            Some(ch) => {
                Err(LexerError::invalid_escape_sequence(format!("\\{}", ch), line, column, self.position))
            }
            None => {
                Err(LexerError::invalid_escape_sequence("\\".to_string(), line, column, self.position))
            }
        }
    }

    /// 读取十六进制转义序列 \xHH
    fn read_hex_escape(&mut self, line: usize, column: usize) -> LexerResult<String> {
        let mut hex = String::new();
        
        for _ in 0..2 {
            if let Some(ch) = self.current_char {
                if ch.is_ascii_hexdigit() {
                    hex.push(ch);
                    self.advance();
                } else {
                    return Err(LexerError::invalid_escape_sequence(format!("\\x{}", hex), line, column, self.position));
                }
            } else {
                return Err(LexerError::invalid_escape_sequence(format!("\\x{}", hex), line, column, self.position));
            }
        }
        
        if let Ok(value) = u8::from_str_radix(&hex, 16) {
            Ok((value as char).to_string())
        } else {
            Err(LexerError::invalid_escape_sequence(format!("\\x{}", hex), line, column, self.position))
        }
    }

    /// 读取Unicode转义序列 \uXXXX 或 \u{XXXXXX}
    fn read_unicode_escape(&mut self, line: usize, column: usize) -> LexerResult<String> {
        let mut hex = String::new();
        let use_braces = self.current_char == Some('{');
        
        if use_braces {
            self.advance(); // 跳过 '{'
            
            while let Some(ch) = self.current_char {
                if ch == '}' {
                    self.advance();
                    break;
                } else if ch.is_ascii_hexdigit() && hex.len() < 6 {
                    hex.push(ch);
                    self.advance();
                } else {
                    return Err(LexerError::invalid_unicode_escape(format!("\\u{{{}}}", hex), line, column, self.position));
                }
            }
        } else {
            // 固定4位十六进制
            for _ in 0..4 {
                if let Some(ch) = self.current_char {
                    if ch.is_ascii_hexdigit() {
                        hex.push(ch);
                        self.advance();
                    } else {
                        return Err(LexerError::invalid_unicode_escape(format!("\\u{}", hex), line, column, self.position));
                    }
                } else {
                    return Err(LexerError::invalid_unicode_escape(format!("\\u{}", hex), line, column, self.position));
                }
            }
        }
        
        if hex.is_empty() {
            return Err(LexerError::invalid_unicode_escape("\\u{}".to_string(), line, column, self.position));
        }
        
        if let Ok(code_point) = u32::from_str_radix(&hex, 16) {
            if let Some(ch) = char::from_u32(code_point) {
                return Ok(ch.to_string());
            }
        }
        
        Err(LexerError::invalid_unicode_escape(
            if use_braces {
                format!("\\u{{{}}}", hex)
            } else {
                format!("\\u{}", hex)
            },
            line,
            column,
            self.position
        ))
    }

    /// 获取下一个Token
    pub fn next_token(&mut self) -> LexerResult<Token> {
        loop {
            self.skip_whitespace();

            if self.current_char == Some('/') && self.peek(1) == Some('/') {
                self.skip_comment();
                continue;
            }

            break;
        }

        let start_pos = self.current_position();

        match self.current_char {
            None => Ok(Token::new(TokenType::EOF, String::new(), start_pos.clone(), start_pos)),
            Some(ch) => {
                // 数字
                if ch.is_ascii_digit() {
                    return self.read_number();
                }

                // 标识符和关键字
                if ch.is_alphabetic() || ch == '_' {
                    // 检查raw字符串
                    if ch == 'r' && self.peek(1) == Some('"') {
                        return self.read_raw_string();
                    }
                    return Ok(self.read_identifier());
                }

                // 字符串
                if ch == '"' {
                    return self.read_string();
                }

                // 字符
                if ch == '\'' {
                    return self.read_char();
                }

                // 运算符和分隔符
                let token = match ch {
                    '+' => {
                        self.advance();
                        if self.current_char == Some('=') {
                            self.advance();
                            Token::new(TokenType::PlusEqual, "+=".to_string(), start_pos, self.current_position())
                        } else {
                            Token::new(TokenType::Plus, "+".to_string(), start_pos, self.current_position())
                        }
                    }
                    '-' => {
                        self.advance();
                        if self.current_char == Some('=') {
                            self.advance();
                            Token::new(TokenType::MinusEqual, "-=".to_string(), start_pos, self.current_position())
                        } else if self.current_char == Some('>') {
                            self.advance();
                            Token::new(TokenType::Arrow, "->".to_string(), start_pos, self.current_position())
                        } else {
                            Token::new(TokenType::Minus, "-".to_string(), start_pos, self.current_position())
                        }
                    }
                    '*' => {
                        self.advance();
                        if self.current_char == Some('=') {
                            self.advance();
                            Token::new(TokenType::StarEqual, "*=".to_string(), start_pos, self.current_position())
                        } else {
                            Token::new(TokenType::Star, "*".to_string(), start_pos, self.current_position())
                        }
                    }
                    '/' => {
                        self.advance();
                        if self.current_char == Some('=') {
                            self.advance();
                            Token::new(TokenType::SlashEqual, "/=".to_string(), start_pos, self.current_position())
                        } else {
                            Token::new(TokenType::Slash, "/".to_string(), start_pos, self.current_position())
                        }
                    }
                    '%' => {
                        self.advance();
                        if self.current_char == Some('=') {
                            self.advance();
                            Token::new(TokenType::PercentEqual, "%=".to_string(), start_pos, self.current_position())
                        } else {
                            Token::new(TokenType::Percent, "%".to_string(), start_pos, self.current_position())
                        }
                    }
                    '=' => {
                        self.advance();
                        if self.current_char == Some('=') {
                            self.advance();
                            Token::new(TokenType::EqualEqual, "==".to_string(), start_pos, self.current_position())
                        } else {
                            Token::new(TokenType::Equal, "=".to_string(), start_pos, self.current_position())
                        }
                    }
                    '!' => {
                        self.advance();
                        if self.current_char == Some('=') {
                            self.advance();
                            Token::new(TokenType::BangEqual, "!=".to_string(), start_pos, self.current_position())
                        } else {
                            Token::new(TokenType::Bang, "!".to_string(), start_pos, self.current_position())
                        }
                    }
                    '<' => {
                        self.advance();
                        if self.current_char == Some('=') {
                            self.advance();
                            Token::new(TokenType::LessEqual, "<=".to_string(), start_pos, self.current_position())
                        } else {
                            Token::new(TokenType::Less, "<".to_string(), start_pos, self.current_position())
                        }
                    }
                    '>' => {
                        self.advance();
                        if self.current_char == Some('=') {
                            self.advance();
                            Token::new(TokenType::GreaterEqual, ">=".to_string(), start_pos, self.current_position())
                        } else {
                            Token::new(TokenType::Greater, ">".to_string(), start_pos, self.current_position())
                        }
                    }
                    '&' => {
                        self.advance();
                        if self.current_char == Some('&') {
                            self.advance();
                            Token::new(TokenType::And, "&&".to_string(), start_pos, self.current_position())
                        } else {
                            Token::new(TokenType::Unknown, "&".to_string(), start_pos, self.current_position())
                        }
                    }
                    '|' => {
                        self.advance();
                        if self.current_char == Some('|') {
                            self.advance();
                            Token::new(TokenType::Or, "||".to_string(), start_pos, self.current_position())
                        } else {
                            Token::new(TokenType::Unknown, "|".to_string(), start_pos, self.current_position())
                        }
                    }
                    '(' => {
                        self.advance();
                        Token::new(TokenType::LeftParen, "(".to_string(), start_pos, self.current_position())
                    }
                    ')' => {
                        self.advance();
                        Token::new(TokenType::RightParen, ")".to_string(), start_pos, self.current_position())
                    }
                    '{' => {
                        self.advance();
                        Token::new(TokenType::LeftBrace, "{".to_string(), start_pos, self.current_position())
                    }
                    '}' => {
                        self.advance();
                        Token::new(TokenType::RightBrace, "}".to_string(), start_pos, self.current_position())
                    }
                    '[' => {
                        self.advance();
                        Token::new(TokenType::LeftBracket, "[".to_string(), start_pos, self.current_position())
                    }
                    ']' => {
                        self.advance();
                        Token::new(TokenType::RightBracket, "]".to_string(), start_pos, self.current_position())
                    }
                    ',' => {
                        self.advance();
                        Token::new(TokenType::Comma, ",".to_string(), start_pos, self.current_position())
                    }
                    ';' => {
                        self.advance();
                        Token::new(TokenType::Semicolon, ";".to_string(), start_pos, self.current_position())
                    }
                    ':' => {
                        self.advance();
                        if self.current_char == Some(':') {
                            self.advance();
                            Token::new(TokenType::DoubleColon, "::".to_string(), start_pos, self.current_position())
                        } else {
                            Token::new(TokenType::Colon, ":".to_string(), start_pos, self.current_position())
                        }
                    }
                    '.' => {
                        self.advance();
                        if self.current_char == Some('.') {
                            self.advance();
                            Token::new(TokenType::DotDot, "..".to_string(), start_pos, self.current_position())
                        } else {
                            Token::new(TokenType::Dot, ".".to_string(), start_pos, self.current_position())
                        }
                    }
                    _ => {
                        self.advance();
                        Token::new(TokenType::Unknown, ch.to_string(), start_pos, self.current_position())
                    }
                };

                Ok(token)
            }
        }
    }

    /// 标记化整个输入
    pub fn tokenize(&mut self) -> LexerResult<Vec<Token>> {
        let mut tokens = Vec::new();
        
        loop {
            let token = self.next_token()?;
            let is_eof = matches!(token.token_type, TokenType::EOF);
            tokens.push(token);
            
            if is_eof {
                break;
            }
        }
        
        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_tracking() {
        let mut lexer = Lexer::new("let x = 42;".to_string());
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].start_pos.line, 1);
        assert_eq!(tokens[0].start_pos.column, 1);
        assert_eq!(tokens[1].start_pos.line, 1);
        assert_eq!(tokens[1].start_pos.column, 5);
    }

    #[test]
    fn test_utf8_identifiers() {
        let mut lexer = Lexer::new("let 变量 = 10;".to_string());
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[1].token_type, TokenType::Identifier);
        assert_eq!(tokens[1].value, "变量");
    }

    #[test]
    fn test_escape_sequences() {
        let mut lexer = Lexer::new(r#""hello\nworld\t""#.to_string());
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token_type, TokenType::String);
        assert_eq!(tokens[0].value, "hello\nworld\t");
    }

    #[test]
    fn test_hex_numbers() {
        let mut lexer = Lexer::new("0xFF 0x10".to_string());
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token_type, TokenType::Integer);
        assert_eq!(tokens[0].value, "0xFF");
    }

    #[test]
    fn test_binary_numbers() {
        let mut lexer = Lexer::new("0b1010 0b11".to_string());
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token_type, TokenType::Integer);
        assert_eq!(tokens[0].value, "0b1010");
    }

    #[test]
    fn test_scientific_notation() {
        let mut lexer = Lexer::new("1e10 3.14e-5".to_string());
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token_type, TokenType::ScientificExponent);
        assert_eq!(tokens[0].value, "1e10");
        assert_eq!(tokens[1].token_type, TokenType::ScientificExponent);
    }

    #[test]
    fn test_compound_assignment() {
        let mut lexer = Lexer::new("+= -= *= /= %=".to_string());
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token_type, TokenType::PlusEqual);
        assert_eq!(tokens[1].token_type, TokenType::MinusEqual);
        assert_eq!(tokens[2].token_type, TokenType::StarEqual);
        assert_eq!(tokens[3].token_type, TokenType::SlashEqual);
        assert_eq!(tokens[4].token_type, TokenType::PercentEqual);
    }

    #[test]
    fn test_raw_string() {
        let mut lexer = Lexer::new(r#"r"hello\nworld""#.to_string());
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token_type, TokenType::String);
        assert_eq!(tokens[0].value, r"hello\nworld");
    }
}