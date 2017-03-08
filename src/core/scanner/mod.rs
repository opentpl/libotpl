use super::token::ascii;
use super::token::Token;


fn is_whitespace(c: u8) -> bool {
    return c == ascii::SP || c == ascii::TB || c == ascii::CR || c == ascii::LF;
}

fn is_lower_letter(c: u8) -> bool {
    return c >= 97u8 && c <= 122u8;
}

fn is_upper_letter(c: u8) -> bool {
    return c >= 65u8 && c <= 90u8;
}

fn is_digit(c: u8) -> bool {
    return c >= 48u8 && c <= 57u8;
}

trait Queue<T> {
    fn offer(&mut self, t: T);
    fn take(&mut self) -> Option<T> ;
}

impl<T> Queue<T> for Vec<T> {
    fn offer(&mut self, t: T) {
        self.insert(0, t);
    }

    fn take(&mut self) -> Option<T> {
        self.pop()
    }
}



#[derive(Debug)]
struct Scanner<'a> {
    line: usize,
    column: usize,
    offset: usize,
    src: &'a [u8],
    stmt_start: &'a [u8],
    stmt_end: &'a [u8],
    // 是否处于OTPL段
    in_stmt: bool,
    // 是否处于原义输出段
    in_literal: bool,
    // 是否处于注释段
    in_comment: bool,
    tok_buf: Vec<Token>,
}
//inLiteral

#[allow(dead_code)]
impl<'a> Scanner<'a> {
    pub fn new(src: &'a [u8], stmt_start: &'a [u8], stmt_end: &'a [u8]) -> Scanner<'a> {
        return Scanner {
            line: 0,
            column: 0,
            offset: 0,
            src: src,
            stmt_start: stmt_start,
            stmt_end: stmt_end,
            in_stmt: false,
            in_literal: false,
            in_comment: false,
            tok_buf: vec![],
        };
    }

    /// 获取处于当前偏移位置的字符。
    fn current(&self) -> u8 {
        self.src[self.offset]
    }
    /// 判断是否可向前。
    fn can_forward(&self) -> bool {
        self.offset + 1 < self.src.len()
    }
    /// 当前偏移位置+1，并处理行标和列标。
    fn forward(&mut self) -> bool {
        if !self.can_forward() {
            return false;
        }
        self.offset += 1;
        self.column += 1;
        if self.current() == ascii::LF {
            if self.assert_match(self.offset + 1, ascii::CR) {
                self.offset += 1;
            }
            self.line += 1;
            self.column = 1;
        } else if self.current() == ascii::CR {
            self.line += 1;
            self.column = 1;
        }
        return true;
    }

    /// 当前偏移位置-1，并处理行标和列标。
    fn back(&mut self) {
        if self.offset - 1 < 0 {
            panic!("超出索引");
        }
        self.offset -= 1;
        self.column -= 1;
        if self.current() == ascii::CR || self.current() == ascii::LF {
            self.line -= 1;

            self.column = 1;
            let mut pos = self.offset;
            while pos >= 0 {
                if self.src[pos] == ascii::CR {
                    // if pos - 1 >= 0 && self.src[pos - 1] == ascii::LF {
                    //
                    // }
                    break;
                } else if self.src[pos] == ascii::LF {
                    break;
                }
                self.column += 1;
                pos -= 1;
            }
        }
    }

    fn assert_match(&self, offset: usize, b: u8) -> bool {
        if offset < 0 || offset >= self.src.len() {
            return false;
        }
        return self.src[offset] == b;
    }
    /// 与当前偏移的下一个字符作比较，如果可用的话。
    fn assert_next(&self, b: u8) -> bool {
        if self.offset + 1 >= self.src.len() {
            return false;
        }
        return self.src[self.offset] == b;
    }
    /// 当前偏移位置+n, n为负数则调用 back() 否则 forward().
    fn seek(&mut self, n: isize) {
        if n < 0 {
            for i in 0..n.abs() {
                self.back()
            }
        } else if n > 0 {
            for i in 0..n {
                self.forward();
            }
        }
    }
    /// 根据当前位置与参数 pos 的差值调用 back() 。
    fn back_pos_diff(&mut self, pos: usize) {
        if pos >= self.offset {
            return;
        }
        let n = (self.offset - pos) as isize;
        self.seek(-n);
    }

    /// 消费掉连续的空白字符串
    fn consume_whitespace(&mut self) {
        while self.can_forward() {
            if is_whitespace(self.current()) {
                self.forward();
            } else {
                break;
            }
        }
    }

    fn match_stmt_start(&mut self) -> Option<Token> {
        let none = Option::None;
        if self.current() != self.stmt_start[0] {
            return none;
        }
        let (line, col) = (self.line, self.column);
        let pos = self.offset;
        for i in 0..self.stmt_start.len() {
            if self.current() != self.stmt_start[i] {
                self.back_pos_diff(pos);
                return none;
            }
            self.forward();
        }
        let mut tmp:Vec<u8> = vec![];
        tmp.extend_from_slice(self.stmt_start);
        return Some(Token::StmtStart(line, col, tmp));
    }

    fn match_stmt_end(&mut self) -> bool {
        let none = Token::None;
        if self.current() != self.stmt_end[0] {
            return false;
        }
        let pos = self.offset;
        for i in 0..self.stmt_start.len() {
            if self.current() != self.stmt_end[i] {
                self.back_pos_diff(pos);
                return false;
            }
            self.forward();
        }
        return true;
    }

    fn scan_stmt(&mut self) -> Token {
        let (line, column) = (self.line, self.column);
        let c = self.current();
        if c == ascii::LSS {
            self.forward();
            if self.assert_next(ascii::EQS) {
                self.forward();
                return Token::LEQ(line, column);
            }
            return Token::LSS(line, column);
        } else if c == ascii::GTR {
            self.forward();
            if self.assert_next(ascii::EQS) {
                self.forward();
                return Token::GEQ(line, column);
            }
            return Token::GTR(line, column);
        }
        return Token::None;
    }

    /// 提取dom标签或属性名称
    fn match_dom_name(&mut self, allowDollarPrefix: bool, allowAtPrefix: bool, allowUnderline: bool) -> Option<Vec<u8>> {
        let c = self.current();
        if !allowDollarPrefix && c == ascii::DLS {
            return Option::None;
        }
        if !allowAtPrefix && c == ascii::ATS {
            return Option::None;
        }
        if !allowUnderline && c == ascii::UND {
            return Option::None;
        }
        let mut buf: Vec<u8> = vec![];
        if is_lower_letter(c)
            || is_upper_letter(c)
            || c == ascii::DLS
            || c == ascii::ATS
            || c == ascii::UND {
            buf.push(c);
        } else {
            return Option::None;
        }
        let pos = self.offset;
        while self.can_forward() {
            self.forward();
            let c = self.current();
            if is_whitespace(c)
                || c == ascii::SLA
                || c == ascii::GTR
                || c == ascii::EQS {
                // 匹配 / > =
                break;
            } else if is_lower_letter(c)
                || is_upper_letter(c)
                || is_digit(c)
                || c == ascii::UND {
                buf.push(c);
                continue;
            }
            self.back_pos_diff(pos);
            return Option::None;
        }
        return Option::Some(buf);
    }

    /// 提取字符串，未找到返回 None
    fn extract_str(&mut self, end: u8) -> Option<Vec<u8>> {
        let mut s: Vec<u8> = vec![];
        let pos = self.offset;
        while self.can_forward() {
            self.forward();
            let c = self.current();
            if c == ascii::BKS {
                // s.push(c); // TODO: 需要带转义符吗？
                if self.can_forward() {
                    self.forward();
                    s.push(self.current());
                }
                continue;
            } else if c == end {
                return Option::Some(s);
            }
            s.push(c);
        }
        self.back_pos_diff(pos);
        return Option::None;
    }


    /// 扫描 dom 节点，并暂存。注意：该方法不自动回退。
    fn scan_dom(&mut self) -> bool {
        //匹配 <
        if self.current() != ascii::LSS {
            return false;
        }
        self.forward();
        if !self.can_forward() {
            return false;
        }
        let (line, column) = (self.line, self.column);
        let tmp = self.match_dom_name(false, false, false);
        if let Option::Some(name) = tmp {
            debug!("dom tag: {:?}", name);
            self.tok_buf.offer(Token::DomTagStart(line, column, name));
        } else {
            return false;
        }

        while self.can_forward() {
            self.consume_whitespace();
            debug!("expected dom attr name first char: {:?}", self.current());
            let (line, column) = (self.line, self.column);
            let tmp = self.match_dom_name(true, true, true);
            if let Option::Some(name) = tmp {
                self.tok_buf.offer(Token::DomTagAttrName(line, column, name));
                // 扫描属性表达式 name="value"
                self.consume_whitespace();
                // 匹配 =
                if self.current() != ascii::EQS {
                    //如果不匹配则视为独立属性
                    continue;
                }
                let (line, column) = (self.line, self.column);
                //吃掉=和空白
                self.forward();
                self.tok_buf.offer(Token::Symbol(line, column, ascii::EQS));
                self.consume_whitespace();
                //匹配字符串
                if self.current() != ascii::QUO {
                    panic!("期望引号 ，找到 {}", self.current());
                }
                let tmp = self.extract_str(ascii::QUO);
                if let Option::Some(s) = tmp {
                    //解析属性值
                    //TODO: 处理扩展语法
                    let mut ts = Scanner::new(&s[..], self.stmt_start, self.stmt_end);
                    ts.line = self.line;
                    loop {
                        let tok = ts.scan();
                        if let Token::None = tok {
                            break;
                        } else {
                            self.tok_buf.offer(tok);
                        }
                    }
                } else {
                    panic!("字符串未结束");
                }
            }
            let (line, column) = (self.line, self.column);
            if self.current() == ascii::SLA && self.assert_next(ascii::GTR) {
                self.seek(2);
                self.tok_buf.offer(Token::DomTagEnd(line, column, vec![ascii::SLA, ascii::GTR]));
                return true;
            } else if self.current() == ascii::GTR {
                self.forward();
                self.tok_buf.offer(Token::DomTagEnd(line, column, vec![ascii::GTR]));
                return true;
            }
            //结束
            self.forward();
        }
        return false;
    }

    pub fn scan(&mut self) -> Token {
        if !self.tok_buf.is_empty() {
            return self.tok_buf.take().unwrap();
        }
        if self.src.len() == 0 || self.offset >= self.src.len() - 1 {
            return Token::None;
        }
        if self.in_stmt {
            return self.scan_stmt();
        }
        let (line, column) = (self.line, self.column);
        let some_token = self.match_stmt_start();
        match some_token {
            Option::None => {},
            Option::Some(token) => {
                self.consume_whitespace();

                if self.current() == ascii::REM {
                    //% 原义输出
                    self.consume_whitespace();
                    if self.match_stmt_end() {
                        self.in_literal = !self.in_literal;
                        return Token::LiteralBoundary(line, column, self.in_literal);
                    }
                } else if self.current() == ascii::SLA && self.assert_match(self.offset + 1, ascii::SLA) {
                    // //单行注释
                    // TODO: 单行注释
                } else if self.current() == ascii::SLA && self.assert_match(self.offset + 1, ascii::MUL) {
                    // /*多行注释
                    // TODO: 多行注释
                }
                self.in_stmt = true;
                // 不转义输出表达式
                //            if self.current() == ascii::NOT
                //                && self.can_forward()
                //                && is_whitespace(self.src[self.offset + 1]) {
                //                // TODO: 多行注释
                //            }
                return token;
            }
        }

        if !self.in_comment && !self.in_literal && self.current() == ascii::LSS {
            let pos = self.offset;
            if self.scan_dom() {
                return self.scan();
            }
            self.back_pos_diff(pos);
        }

        let begin_char = self.stmt_start[0];
        let mut buf: Vec<u8> = vec![];

        while self.can_forward() {
            if self.current() == begin_char {
                let pos = self.offset;
                let some_token = self.match_stmt_start();
                match some_token {
                    Option::None => {},
                    _ => {
                        self.back_pos_diff(pos);
                        break;
                    }
                }
            }
            buf.push(self.current());
            self.forward();
        }
        if self.in_comment {
            return Token::Comments(line, column, buf);
        }
        if self.in_literal {
            return Token::Literal(line, column, buf);
        }
        return Token::Data(line, column, buf);
    }
}


#[test]
fn test_scan() {
    let mut eof = false;
    let mut scanner = Scanner::new("<div id=\"te\\\"st\">".as_bytes(), "{{".as_bytes(), "}}".as_bytes());
    'outer: loop {
        let token = scanner.scan();
        match token {
            Token::None => { break 'outer; },
            _ => {
                debug!("scanned token: {:?}", token);
            }
        }
    }
}






