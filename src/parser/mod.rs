use crate::lexer::lexer::Lexer;
use crate::lexer::token::{Token, EOF_TOKEN};
use crate::parser::program::{Expression, Ident, Precedence, Program, Statement};
use std::fmt::{Debug, Display, Formatter};

pub mod program;

#[cfg(test)]
mod test;

pub type Result<T> = std::result::Result<T, ParseError>;

pub struct Parser {
    l: Lexer,
    cur_token: Token,
    peek_token: Token,
}

pub struct ParseError {
    info: String,
}

impl From<&str> for ParseError {
    fn from(s: &str) -> Self {
        ParseError { info: s.to_owned() }
    }
}

impl From<String> for ParseError {
    fn from(s: String) -> Self {
        ParseError { info: s }
    }
}

impl Debug for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.info)
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.info)
    }
}

impl std::error::Error for ParseError {}

impl Parser {
    pub fn new(l: Lexer) -> Self {
        let mut ret = Parser {
            l,
            cur_token: EOF_TOKEN,
            peek_token: EOF_TOKEN,
        };
        ret.next_token();
        ret.next_token();

        ret
    }

    pub fn next_token(&mut self) {
        std::mem::swap(&mut self.cur_token, &mut self.peek_token);
        self.peek_token = self.l.next_token();
    }

    pub fn expect_peek(&mut self, token: Token) -> bool {
        if self.peek_token == token {
            self.next_token();
            true
        } else {
            log::debug!(
                "{}:{} parser error: expect next token to be {:?}, got {:?} instead",
                file!(),
                line!(),
                token,
                self.peek_token
            );
            false
        }
    }

    pub fn parse_program(&mut self) -> Result<Program> {
        let mut ret = Program::default();
        loop {
            // println!("[parse loop] current token is {:?}", self.cur_token);
            if self.cur_token.is_eof() {
                break;
            }

            let statement = self.parse_statement()?;
            ret.statements.push(statement);

            self.next_token();
        }
        Ok(ret)
    }

    fn parse_statement(&mut self) -> Result<Statement> {
        match self.cur_token {
            Token::Let => self.parse_let_statement(),
            Token::Return => self.parse_return_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_let_statement(&mut self) -> Result<Statement> {
        if let Token::Ident(_) = &self.peek_token {
            self.next_token();
        }
        let identifier = self.parse_identifier()?;

        if !self.expect_peek(Token::Assign) {
            return Err("no equal sign!".into());
        }

        self.next_token();

        let value = self.parse_expression(Precedence::Lowest)?;
        if self.peek_token == Token::Semicolon {
            self.next_token();
        }
        Ok(Statement::LetStatement(identifier, value))
    }

    fn parse_return_statement(&mut self) -> Result<Statement> {
        self.next_token();

        let ret = self.parse_expression(Precedence::Lowest)?;
        if self.peek_token == Token::Semicolon {
            self.next_token();
        }

        Ok(Statement::ReturnStatement(ret))
    }

    fn parse_expression_statement(&mut self) -> Result<Statement> {
        let ret = self.parse_expression(Precedence::Lowest)?;
        if self.peek_token == Token::Semicolon {
            self.next_token();
        }

        Ok(Statement::ExpressionStatement(ret))
    }

    fn parse_identifier(&mut self) -> Result<Ident> {
        match &self.cur_token {
            Token::Ident(v) => Ok(Ident(v.clone())),
            _ => Err("not a ident token".into()),
        }
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Result<Expression> {
        // prefix
        let mut left = match &self.cur_token {
            Token::Ident(_) => {
                let ident = self.parse_identifier()?;
                Ok(Expression::Identifier(ident))
            }
            Token::Int(_) => self.parse_int_literal(),
            Token::Bool(_) => self.parse_bool_literal(),
            Token::String(_) => self.parse_string_literal(),
            Token::Bang | Token::Minus => {
                if precedence > Precedence::Prefix {
                    Err(format!(
                        "'(' expected after prefix '{}'",
                        &self.cur_token.to_string()
                    )
                    .into())
                } else {
                    self.parse_prefix_expression()
                }
            }
            Token::LParen => self.parse_grouped_expression(),
            Token::If => self.parse_if_expression(),
            Token::Function => self.parse_function_literal(),
            Token::LBracket => self.parse_array_literal(),
            Token::LBrace => self.parse_hash_literal(),
            _ => Err(format!("no prefix parse function for {:?}", self.cur_token)
                .as_str()
                .into()),
        }?;

        // infix
        loop {
            if self.peek_token == Token::Semicolon
                || precedence >= Precedence::from_token(&self.peek_token)
            {
                break;
            }

            self.next_token();
            match self.cur_token {
                Token::Eq
                | Token::NotEq
                | Token::LT
                | Token::GT
                | Token::Plus
                | Token::Minus
                | Token::Slash
                | Token::Asterisk => left = self.parse_infix_expression(left)?,
                Token::LParen => left = self.parse_call_expression(left)?,
                Token::LBracket => left = self.parse_index_expression(left)?,
                _ => return Ok(left),
            };
        }

        Ok(left)
    }

    fn parse_int_literal(&self) -> Result<Expression> {
        if let Token::Int(v) = self.cur_token {
            Ok(Expression::IntLiteral(v))
        } else {
            Err("Token::Int not found".into())
        }
    }

    fn parse_bool_literal(&self) -> Result<Expression> {
        if let Token::Bool(v) = self.cur_token {
            Ok(Expression::BoolLiteral(v))
        } else {
            Err("Token::Bool not found".into())
        }
    }

    fn parse_string_literal(&self) -> Result<Expression> {
        if let Token::String(v) = &self.cur_token {
            Ok(Expression::StringLiteral(v.clone()))
        } else {
            Err("Token::String not found".into())
        }
    }

    fn parse_prefix_expression(&mut self) -> Result<Expression> {
        let token = self.cur_token.clone();
        let precedence = match &token {
            Token::Minus => Precedence::Prefix.add(1),
            _ => Precedence::Prefix,
        };
        self.next_token();

        let right = self.parse_expression(precedence)?;
        Ok(Expression::PrefixExpression(token, Box::new(right)))
    }

    fn parse_infix_expression(&mut self, left: Expression) -> Result<Expression> {
        let precedence = Precedence::from_token(&self.cur_token);
        let token = self.cur_token.clone();
        self.next_token();

        let right = match &token {
            // to make '+' right-associate
            // Token::Plus => self.parse_expression(precedence.sub(1))?,
            _ => self.parse_expression(precedence)?,
        };
        Ok(Expression::InfixExpression(
            Box::new(left),
            token,
            Box::new(right),
        ))
    }

    fn parse_grouped_expression(&mut self) -> Result<Expression> {
        self.next_token();

        let exp = self.parse_expression(Precedence::Lowest)?;

        if !self.expect_peek(Token::RParen) {
            return Err("Right parentheses expected".into());
        }

        return Ok(exp);
    }

    fn parse_if_expression(&mut self) -> Result<Expression> {
        if !self.expect_peek(Token::LParen) {
            return Err("'(' expected after 'if'.".into());
        }
        self.next_token();
        let condition = self.parse_expression(Precedence::Lowest)?;

        if !self.expect_peek(Token::RParen) {
            return Err("')' expected after if condition expression".into());
        }

        if !self.expect_peek(Token::LBrace) {
            return Err("'{' expected for block.".into());
        }

        let consequence = self.parse_block_statement()?;

        let alternative = match self.peek_token {
            _ => vec![],
        };
        return Ok(Expression::IfExpression(
            Box::new(condition),
            consequence,
            alternative,
        ));
    }

    fn parse_block_statement(&mut self) -> Result<Vec<Statement>> {
        self.next_token(); // LBrace

        let mut ret = vec![];
        while self.cur_token != Token::RBrace {
            let statement = self.parse_statement()?;
            ret.push(statement);

            self.next_token();
        }
        Ok(ret)
    }

    fn parse_function_literal(&mut self) -> Result<Expression> {
        // TODO! 支持function名称
        if !self.expect_peek(Token::LParen) {
            return Err("'(' expected for function expression".into());
        }

        let params = self.parse_function_parameters()?;

        if !self.expect_peek(Token::LBrace) {
            return Err("'{' expected for function body.".into());
        }

        let sts = self.parse_block_statement()?;
        Ok(Expression::FunctionExpression(params, sts))
    }

    fn parse_array_literal(&mut self) -> Result<Expression> {
        let elements = self.parse_expression_list(&Token::RBracket)?;

        if !self.expect_peek(Token::RBracket) {
            return Err("']' expected for array definition.".into());
        }

        Ok(Expression::ArrayLiteral(elements))
    }

    fn parse_hash_literal(&mut self) -> Result<Expression> {
        let mut ret: Vec<(Expression, Expression)> = Default::default();
        while self.peek_token != Token::RBrace {
            self.next_token();
            let key = self.parse_expression(Precedence::Lowest)?;

            if !self.expect_peek(Token::Colon) {
                return Err("':' expected in Hash element.".into());
            }

            self.next_token();
            let value = self.parse_expression(Precedence::Lowest)?;
            ret.push((key, value));

            if self.peek_token != Token::RBrace && self.peek_token != Token::Comma {
                return Err("'}' or ',' expected in Hash element.".into());
            }
        }

        if !self.expect_peek(Token::RBrace) {
            return Err("'}' expected for Hash end.".into());
        }

        Ok(Expression::HashLiteral(ret))
    }

    fn parse_expression_list(&mut self, end: &Token) -> Result<Vec<Expression>> {
        let mut ret = vec![];

        if self.peek_token.eq(end) {
            self.next_token();
            return Ok(ret);
        }

        self.next_token();
        ret.push(self.parse_expression(Precedence::Lowest)?);

        while self.peek_token.eq(&Token::Comma) {
            self.next_token(); // comma
            self.next_token(); // next argument
            ret.push(self.parse_expression(Precedence::Lowest)?);
        }

        Ok(ret)
    }

    fn parse_function_parameters(&mut self) -> Result<Vec<Ident>> {
        let mut ret = vec![];
        self.next_token();

        // 没有参数的情况
        if self.cur_token == Token::RParen {
            return Ok(ret);
        }

        loop {
            if let Token::Ident(v) = &self.cur_token {
                ret.push(Ident(v.clone()));
            }

            if self.peek_token != Token::Comma {
                break;
            }

            self.next_token(); // comma
            self.next_token(); // next ident
        }

        if !self.expect_peek(Token::RParen) {
            return Err("')' expected for function parameters expression.".into());
        }

        Ok(ret)
    }

    fn parse_call_expression(&mut self, function: Expression) -> Result<Expression> {
        Ok(Expression::CallExpression(
            Box::new(function),
            self.parse_call_arguments()?,
        ))
    }

    fn parse_call_arguments(&mut self) -> Result<Vec<Expression>> {
        let ret = self.parse_expression_list(&Token::RParen)?;

        if !self.expect_peek(Token::RParen) {
            return Err("')' expected for function call.".into());
        }

        Ok(ret)
    }

    fn parse_index_expression(&mut self, left: Expression) -> Result<Expression> {
        self.next_token();
        let index = self.parse_expression(Precedence::Lowest)?;

        if !self.expect_peek(Token::RBracket) {
            return Err("']' expected for index end.".into());
        }
        Ok(Expression::IndexExpression(Box::new(left), Box::new(index)))
    }
}
