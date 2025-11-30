
use std::iter::Peekable;

use logos::{Lexer, Logos};

use crate::{error::{PakError, PakResult, PqlError, PqlResult}, query::PakQueryExpression, value::PakValue};

//==============================================================================================
//        PQL Tokens
//==============================================================================================

#[derive(Logos, Debug, PartialEq, PartialOrd)]
#[logos(skip r"[ \t\n\f]+")] // Ignore this regex pattern between tokens
pub(crate) enum PqlToken {
    #[token("=")]
    Eq,
    #[token("<")]
    Less,
    #[token("<=")]
    LessEq,
    #[token(">")]
    Greater,
    #[token(">=")]
    GreaterEq,
    #[token("|")]
    Or,
    #[token("&")]
    And,
    #[regex("|[ ]+\\(")]
    GroupStartOr,
    #[regex("&[ ]+\\(")]
    GroupStartAnd,
    #[token("(")]
    GroupStart,
    #[token(")")]
    GroupEnd,
    #[regex("[a-zA-Z_][a-zA-Z0-9_-]+", text)]
    Text(String),
    #[regex("[0-9]+", int)]
    #[regex("[0-9]+i", int)]
    Int(i64),
    #[regex("[0-9]+u", uint)]
    Uint(u64),
    #[regex("[0-9]+.[0-9]+", float)]
    #[regex("[0-9]+f", float)]
    Float(f64),
}

impl PqlToken {
    pub fn get_text(&self) -> Option<&String> {
        let PqlToken::Text(text) = self else { return None };
        Some(text)
    }
}

fn text(lex : &mut Lexer<PqlToken>) -> Option<String> {
    Some(lex.slice().to_string())
}

fn int(lex : &mut Lexer<PqlToken>) -> Option<i64> {
    lex.slice().parse().ok()
}

fn uint(lex : &mut Lexer<PqlToken>) -> Option<u64> {
    lex.slice().parse().ok()
}

fn float(lex : &mut Lexer<PqlToken>) -> Option<f64> {
    lex.slice().parse().ok()
}


//==============================================================================================
//        Pql parsing Common
//==============================================================================================

type TokenResult = Result<PqlToken, ()>;

enum Binary<A, B> {
    First(A),
    Second(B)
}

fn next_is<I : Iterator<Item =TokenResult>>(lexer : &mut Peekable<I>, token : &PqlToken) -> PqlResult<bool> {
    let Some(Ok(t)) = lexer.peek() else { return Err(PqlError::EndOfFile.into()) };
    Ok(token == t)
}

//==============================================================================================
//        Query
//==============================================================================================

#[derive(Debug)]
enum PqlQuery {
    Expression(Box<PqlExpression>),
    Group(Box<PqlGroup>)
}

fn parse_query<I : Iterator<Item =TokenResult>>(lexer : &mut Peekable<I>) -> PqlResult<PqlQuery> {
    let group = parse_group(lexer);
    match group {
        Ok(group) => return Ok(PqlQuery::Group(Box::new(group))),
        Err(PqlError::NoMatch) => {},
        Err(err) => return Err(err), 
    };
    let expression = parse_expression(lexer);
    match expression {
        Ok(expression) => Ok(PqlQuery::Expression(Box::new(expression))),
        Err(err) => Err(err),
    }
} 

//==============================================================================================
//        PqlGroup
//==============================================================================================

#[derive(Debug)]
struct PqlGroup(PqlQuery);

fn parse_group<I : Iterator<Item =TokenResult>>(lexer : &mut Peekable<I>) -> PqlResult<PqlGroup> {
    if !next_is(lexer, &PqlToken::GroupStart)? { return Err(PqlError::NoMatch) } 
    lexer.next();
    let query = parse_query(lexer)?;
    if !next_is(lexer, &PqlToken::GroupEnd)? { return Err(PqlError::UnexpectedToken(lexer.next().unwrap().unwrap(), ")".to_string())) }
    lexer.next();
    Ok(PqlGroup(query))
}

//==============================================================================================
//        Expression
//==============================================================================================

#[derive(Debug)]
struct PqlExpression {
    first : PqlStatement,
    second : Option<(PqlToken, PqlQuery)>
}

fn parse_expression<I : Iterator<Item =TokenResult>>(lexer : &mut Peekable<I>) -> PqlResult<PqlExpression> {
    let first = parse_statement(lexer)?;
    if !(next_is(lexer, &PqlToken::Or)? || next_is(lexer, &PqlToken::And)?) { return Ok(PqlExpression { first, second: None }) }
    let Some(Ok(op)) = lexer.next() else { return Ok(PqlExpression { first, second: None })};
    let second = parse_query(lexer)?;
    Ok(PqlExpression { first, second : Some((op, second)) })
}

//==============================================================================================
//        Statement
//==============================================================================================

#[derive(Debug)]
struct PqlStatement {
    key : String,
    op : PqlToken,
    value : PakValue
}

fn parse_statement<I : Iterator<Item =TokenResult>>(lexer : &mut Peekable<I>) -> PqlResult<PqlStatement> {
    let key = parse_text(lexer)?;
    let op = parse_statement_op(lexer)?;
    let value = parse_value(lexer)?;
    Ok(PqlStatement { key, op, value })
}

//==============================================================================================
//        Value Parse
//==============================================================================================

fn parse_value<I : Iterator<Item =TokenResult>>(lexer : &mut Peekable<I>) -> PqlResult<PakValue> {
    if !check_value(lexer)? { return Err(PqlError::NoMatch) }
    let Some(Ok(first_text)) = lexer.next() else { return Err(PqlError::EndOfFile) };
    let value = match first_text {
        PqlToken::Text(value) => PakValue::String(value),
        PqlToken::Int(value) => PakValue::Int(value),
        PqlToken::Uint(value) => PakValue::Uint(value),
        PqlToken::Float(value) => PakValue::Float(value.to_bits()),
        _ => { unreachable!() }
    };
    Ok(value)
}

fn check_value<I : Iterator<Item =TokenResult>>(lexer : &mut Peekable<I>) -> PqlResult<bool> {
    let Some(Ok(next)) = lexer.peek() else { return Err(PqlError::EndOfFile) };
    Ok(matches!(next, PqlToken::Text(_) | PqlToken::Float(_) | PqlToken::Int(_) | PqlToken::Uint(_)))
}

//==============================================================================================
//        Text
//==============================================================================================

fn parse_text<I : Iterator<Item =TokenResult>>(lexer : &mut Peekable<I>) -> PqlResult<String> {
    if !check_text(lexer)? { return Err(PqlError::NoMatch) }
    let Some(Ok(first_text)) = lexer.next() else { return Err(PqlError::EndOfFile) };
    let Some(text) = first_text.get_text() else { return Err(PqlError::NoMatch) };
    Ok(text.clone())
}

fn check_text<I : Iterator<Item =TokenResult>>(lexer : &mut Peekable<I>) -> PqlResult<bool> {
    let Some(Ok(next)) = lexer.peek() else { return Err(PqlError::EndOfFile) };
    Ok(matches!(next, PqlToken::Text(_)))
}

//==============================================================================================
//        Statement Op
//==============================================================================================

fn parse_statement_op<I : Iterator<Item =TokenResult>>(lexer : &mut Peekable<I>) -> PqlResult<PqlToken> {
    if !check_statement_op(lexer)? { return Err(PqlError::NoMatch) }
    Ok(lexer.next().unwrap().unwrap())
}

fn check_statement_op<I : Iterator<Item =TokenResult>>(lexer : &mut Peekable<I>) -> PqlResult<bool> {
    let Some(Ok(next)) = lexer.peek() else { return Err(PqlError::EndOfFile) };
    Ok(matches!(next, PqlToken::Eq | PqlToken::Less | PqlToken::Greater | PqlToken::LessEq | PqlToken::GreaterEq))
}


#[cfg(test)]
mod test {
    use logos::Lexer;

    use crate::{query::pql::{parse_query, parse_statement, PqlToken}, value::PakValue};

    #[test]
    fn pql_parse_query() {
        let pql = "(age <= 25 | name = John) & last_name = Doe";
        let mut lexer = Lexer::<PqlToken>::new(pql).peekable();
        let query = parse_query(&mut lexer);
        println!("Query {query:?}")
    }
    
    #[test]
    fn pql_parse_statement() {
        let pql = "age <= 25";
        let mut lexer = Lexer::<PqlToken>::new(pql).peekable();
        let stmt = parse_statement(&mut lexer).unwrap();
        assert_eq!(stmt.key, "age");
        assert_eq!(stmt.op, PqlToken::LessEq);
        assert_eq!(stmt.value, PakValue::Int(25));
    }
}