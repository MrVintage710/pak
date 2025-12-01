
use std::iter::Peekable;

use logos::{Lexer, Logos};

use crate::{error::{PakResult, PqlError, PqlResult}, query::{PakQuery, PakQueryExpression, PakQueryIntersection, PakQueryUnion}, value::PakValue};

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
    #[regex("[a-zA-Z_]([a-zA-Z0-9_-]+)?", text)]
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
//        Parse Function
//==============================================================================================

pub fn pql(source : &str) -> PakResult<Box<dyn PakQueryExpression>> {
    let mut lexer = Lexer::new(source).peekable();
    match PqlQuery::parse(&mut lexer) {
        Ok(query) => {Ok(query.eval())},
        Err(error) => Err(error.into()),
    }
}

//==============================================================================================
//        Pql parsing Common
//==============================================================================================

type TokenResult = Result<PqlToken, ()>;

fn next_is<I : Iterator<Item =TokenResult>>(lexer : &mut Peekable<I>, token : &PqlToken) -> PqlResult<bool> {
    let Some(Ok(t)) = lexer.peek() else { return Err(PqlError::EndOfFile.into()) };
    Ok(token == t)
}

fn next_is_or_end<I : Iterator<Item =TokenResult>>(lexer : &mut Peekable<I>, token : &PqlToken) -> PqlResult<bool> {
    let Some(Ok(t)) = lexer.peek() else { return Ok(false) };
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

impl PqlQuery {
    fn parse<I : Iterator<Item =TokenResult>>(lexer : &mut Peekable<I>) -> PqlResult<PqlQuery> {
        let group = PqlGroup::parse(lexer);
        match group {
            Ok(group) => return Ok(PqlQuery::Group(Box::new(group))),
            Err(PqlError::NoMatch) => {},
            Err(err) => return Err(err), 
        };
        let expression = PqlExpression::parse(lexer);
        match expression {
            Ok(expression) => Ok(PqlQuery::Expression(Box::new(expression))),
            Err(err) => Err(err),
        }
    }
    
    fn eval(self) -> Box<dyn PakQueryExpression> {
        match self {
            PqlQuery::Expression(pql_expression) => pql_expression.eval(),
            PqlQuery::Group(pql_group) => pql_group.eval(),
        }
    }
}


//==============================================================================================
//        PqlGroup
//==============================================================================================

#[derive(Debug)]
struct PqlGroup(PqlQuery);

impl PqlGroup {
    fn parse<I : Iterator<Item =TokenResult>>(lexer : &mut Peekable<I>) -> PqlResult<PqlGroup> {
        if !next_is_or_end(lexer, &PqlToken::GroupStart)? { return Err(PqlError::NoMatch) }
        lexer.next();
        let query = PqlQuery::parse(lexer)?;
        if !next_is(lexer, &PqlToken::GroupEnd)? { return Err(PqlError::UnexpectedToken(lexer.next().unwrap().unwrap(), ")".to_string())) }
        lexer.next();
        Ok(PqlGroup(query))
    }
    
    fn eval(self) -> Box<dyn PakQueryExpression> {
        self.0.eval()
    }
}


//==============================================================================================
//        Expression
//==============================================================================================

#[derive(Debug)]
struct PqlExpression {
    first : PqlStatement,
    second : Option<(PqlToken, PqlQuery)>
}

impl PqlExpression {
    fn parse<I : Iterator<Item =TokenResult>>(lexer : &mut Peekable<I>) -> PqlResult<PqlExpression> {
        let first = PqlStatement::parse(lexer)?;
        if !(next_is_or_end(lexer, &PqlToken::Or)? || next_is_or_end(lexer, &PqlToken::And)?) { return Ok(PqlExpression { first, second: None }) }
        let Some(Ok(op)) = lexer.next() else { return Ok(PqlExpression { first, second: None })};
        let second = PqlQuery::parse(lexer)?;
        Ok(PqlExpression { first, second : Some((op, second)) })
    }
    
    fn eval(self) -> Box<dyn PakQueryExpression> {
        let first = self.first.eval();
        if let Some((op, second)) = self.second {
            let second = second.eval();
            match op {
                PqlToken::And => Box::new(PakQueryIntersection::new(first, second)),
                PqlToken::Or => Box::new(PakQueryUnion::new(first, second)),
                _ => unreachable!()
            }
        } else {
            Box::new(first)
        }
    }
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

impl PqlStatement {
    fn parse<I : Iterator<Item =TokenResult>>(lexer : &mut Peekable<I>) -> PqlResult<PqlStatement> {
        let key = parse_text(lexer)?;
        let op = parse_statement_op(lexer)?;
        let value = parse_value(lexer)?;
        Ok(PqlStatement { key, op, value })
    }
    
    fn eval(self) -> Box<dyn PakQueryExpression> {
        let query = match self.op {
            PqlToken::Eq => PakQuery::Equal(self.key, self.value),
            PqlToken::Less => PakQuery::LessThan(self.key, self.value),
            PqlToken::LessEq => PakQuery::LessThanEqual(self.key, self.value),
            PqlToken::Greater => PakQuery::GreaterThan(self.key, self.value),
            PqlToken::GreaterEq => PakQuery::GreaterThanEqual(self.key, self.value),
            _ => unreachable!()
        };
        Box::new(query)
    }
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

    use crate::{index::PakIndexIdentifier, query::pql::{PqlExpression, PqlQuery, PqlStatement, PqlToken}, test::{build_data_base, Person, Pet}, value::PakValue};

    #[test]
    fn pql_parse_query() {
        let pql = "(age <= 25 | name = John) & last_name = Doe";
        let mut lexer = Lexer::<PqlToken>::new(pql).peekable();
        let query = PqlQuery::parse(&mut lexer);
    }
    
    #[test]
    fn pql_parse_statement() {
        let pql = "age <= 25";
        let mut lexer = Lexer::<PqlToken>::new(pql).peekable();
        let stmt = PqlStatement::parse(&mut lexer).unwrap();
        assert_eq!(stmt.key, "age");
        assert_eq!(stmt.op, PqlToken::LessEq);
        assert_eq!(stmt.value, PakValue::Int(25));
    }
    
    #[test]
    fn pql_parse_expression() {
        let pql = "age <= 20 | name >= J";
        let mut lexer = Lexer::<PqlToken>::new(pql).peekable();
        let expr = PqlExpression::parse(&mut lexer).unwrap();
        println!("{expr:?}")
    }
    
    #[test]
    fn pql_query_database() {
        let (pak, _, _) = build_data_base();
        let pql = "first_name = John & age <= 25";
        let (people, pets) = pak.query_sql::<(Person, Pet)>(pql).unwrap();
        let (other_people, other_pets) = pak.query::<(Person, Pet)>("first_name".equals("John") & "age".greater_than_or_equal(25)).unwrap();
        println!("Results for `{pql}`:\nPeople {people:#?}\nPets {pets:#?}");
        println!("Other Results:\nPeople {other_people:#?}\nPets {other_pets:#?}");
    }
}