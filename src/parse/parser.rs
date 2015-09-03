///  Program for testing and playing with the parser
///

use std::iter::Iterator;
use super::ast::{Query, DefStmt, CreateStmt, DropStmt, CreateTableStmt, AltStmt,
                 AlterTableStmt, AlterOp, ColumnInfo, SqlType};
use super::token::TokenSpan;
use super::lex::Lexer;
use std::mem::swap;
use super::token::Token;
use std::any::Any;


//TODO: Replace with import!!
#[derive(Debug, Clone)]
pub struct Span {
    pub lo: usize,
    pub hi: usize,
}

#[derive(PartialEq)]
// Keywords that can occour in SQL syntax
pub enum Keyword {
    // 1st level keywords
    // data definition keywords
    Create,
    Drop,
    Alter,

    // data manipulation keywords
    Select,
    Update,
    Insert,
    Delete,


    // 2nd level keywords
    Table,
    View,
    Column,


    // 3rd level keywords
    From,
    Where,
    Modify,
    Add
}



// the parser needs a Lexer that iterates through the query
pub struct Parser<'a>{
    lexiter: Lexer<'a>,

    last: Option<TokenSpan>,
    // the current token given by the lexer
    curr: Option<TokenSpan>,
    // next token
    peek: Option<TokenSpan>
}

impl<'a> Parser<'a>{

    /// Constructs a Parser for the given query.
    pub fn create(query: &'a str) -> Parser<'a>{
        let l = Lexer::from_query(query);
        let mut p= Parser{lexiter:l,last:None,curr:None,peek:None};
        // Sets initial position of lexer and curr/peek
        p.lexer_next();
        p.lexer_next();
        p
    }

    /// Parses the given query into an AST
    pub fn parse(&mut self) -> Result<Query, ParseError>{
        //deletes Whitespaces in the beginning of Query
        match self.expect_token(&[Token::Whitespace]){
            Ok(&Token::Whitespace) => self.lexer_next(),
            _=>(),
        }
        // first token is checked if it's a keyword using expect_keyword()
        let querytype = self.expect_keyword(&[Keyword::Create,Keyword::Drop, Keyword::Alter])
            .map_err(|e| match e {
                ParseError::EofError => ParseError::EmptyQueryError,
                _ => e,
            });
        // returns high-level AST or Error
        match try!(querytype) {

            // Different Query-types are matched and returned

            // Create-Query
            Keyword::Create => return Ok(Query::DefStmt(DefStmt::Create(try!(self.parse_create_stmt())))),
            // Drop-Query
            Keyword::Drop => return Ok(Query::DefStmt(DefStmt::Drop(try!(self.parse_drop_stmt())))),
            // Alter-Query
            Keyword::Alter => return Ok(Query::DefStmt(DefStmt::Alter(try!(self.parse_alt_stmt())))),

            // Unknown Error
            _=> return Err(ParseError::UnknownError)
        }
    }

    // sets next position for the lexer
    fn lexer_next(&mut self){
        swap(&mut self.last, &mut self.curr);  //  last = curr
        swap(&mut self.curr, &mut self.peek);  //  curr = peek
        self.peek = self.lexiter.next();
    }

    // Parses the tokens for create-syntax
    fn parse_create_stmt(&mut self) -> Result<CreateStmt, ParseError> {
        self.skip_whitespace();

        match try!(self.expect_keyword(&[Keyword::Table])) {
            // Create the table subtree
            Keyword::Table=> return {try!(self.skip_whitespace());Err(ParseError::DebugError(try!(self.expect_word())))},
            // Create the view subtree
            // Create .....

            // Unknown parsing error
            _=> return Err(ParseError::UnknownError),
        };

    }

    // Parses tokens for alter statement
    fn parse_alt_stmt(&mut self) -> Result<AltStmt, ParseError> {
        try!(self.skip_whitespace());
        
        match try!(self.expect_keyword(&[Keyword::Table])) {
            Keyword::Table=> return Ok(AltStmt::Table(try!(self.parse_alter_stmt()))),

            // Unknown parsing error
            _=> return Err(ParseError::UnknownError),
        };
    }

    // Parses table to modify and subsequent operations
    fn parse_alter_stmt(&mut self) -> Result<AlterTableStmt, ParseError> {
        try!(self.skip_whitespace());

        let mut alt_table_stmt = AlterTableStmt {tid: try!(self.expect_word()), op: try!(self.parse_alter_op())};
        Ok(alt_table_stmt)

    }

    // Parses operations applied on selected table including tablename and datatype if necessary
    fn parse_alter_op(&mut self) -> Result<AlterOp, ParseError> {
        try!(self.skip_whitespace());

        match try!(self.expect_keyword(&[Keyword::Add, Keyword::Drop, Keyword::Modify])){
            Keyword::Add => return Ok(AlterOp::Add(try!(self.parse_column_info()))),
            Keyword::Drop => return Ok(AlterOp::Drop(try!(self.expect_word()))),
            Keyword::Modify => return Ok(AlterOp::Modify(try!(self.parse_column_info()))),
            _ => return Err(ParseError::UnknownError),
        }
    }


    // Utility function to parse metadata of columns
    fn parse_column_info(&mut self) -> Result<ColumnInfo, ParseError> {
        try!(self.skip_whitespace());
        let mut column_id = try!(self.expect_word());
        try!(self.skip_whitespace());
        //todo insert expect_datatype fn
        let mut column_info = ColumnInfo{cid: column_id, datatype: SqlType::Int};
        Ok(column_info)
    }


    // Parses the tokens for drop statement
    fn parse_drop_stmt(&mut self) -> Result<DropStmt, ParseError> {
        try!(self.skip_whitespace());

        match try!(self.expect_keyword(&[Keyword::Table])) {
            Keyword::Table => return {try!(self.skip_whitespace()); Ok(DropStmt::Table(try!(self.expect_word())))},
            _=> return Err(ParseError::UnknownError),
        };
    }




    // matches current token against any keyword and checks if it is one of the expected keywords
    fn expect_keyword(&self,expected_keywords: &[Keyword]) -> Result<Keyword, ParseError> {
        let mut found_keyword;
        let mut span_lo;
        let mut span_hi;
        {
            // checks if token non or some
            let token = match self.curr {
                None => return Err(ParseError::EofError),
                // in case of som: return reference to token
                Some(ref token) => token,
            };

            span_lo=token.span.lo;
            span_hi=token.span.hi;

            // checks whether token is a word
            let word = match token.tok {
                Token::Word(ref s) => s,
                _=>return Err(ParseError::NotAKeyword(Span {lo: span_lo , hi: span_hi}))
            };

            // checks if word is a keyword
            found_keyword = match &word[..] {
                "create" => Keyword::Create,
                "drop" => Keyword::Drop,
                "table" => Keyword::Table,
                "view" => Keyword::View,
                "alter" => Keyword::Alter,
                "update" => Keyword::Update,
                "select" => Keyword::Select,
                "insert" => Keyword::Insert,
                "delete" => Keyword::Delete,
                "modify" => Keyword::Modify,
                "add" => Keyword::Add,
                "column" => Keyword::Column,
                _=>return Err(ParseError::NotAKeyword(Span {lo: span_lo , hi: span_hi})),

            };
        }
        // checks if keyword is expected keyword
        if(expected_keywords.contains(&found_keyword)){
            return Ok((found_keyword));
        }else{
            return Err(ParseError::WrongKeyword(Span {lo: span_lo , hi: span_hi}));
        }
    }


    fn expect_word(&self) -> Result<String, ParseError>{
        let mut found_word;
        let mut span_lo;
        let mut span_hi;
        {
            // checks if token non or some
            let token = match self.curr {
                None => return Err(ParseError::EofError),
                // in case of som: return reference to token
                Some(ref token) => token,
            };

            span_lo=token.span.lo;
            span_hi=token.span.hi;

            // checks whether token is a word
            found_word = match token.tok {
                Token::Word(ref s) => s,
                _=>return Err(ParseError::NotAWord(Span {lo: span_lo , hi: span_hi}))
            };

        }
        return Ok(found_word.to_string());


    }


    fn expect_token(& self,expected_tokens: &[Token]) -> Result<&Token, ParseError>{


            let token = match self.curr {
                None => return Err(ParseError::EofError),
                // in case of some: return reference to token
                Some(ref token) => token,
            };


        if(expected_tokens.contains(&(token.tok))){
            return Ok(&token.tok);
        }else{
            return Err(ParseError::WrongToken(Span {lo: token.span.lo, hi: token.span.hi}))
        }
    }


    fn skip_whitespace(&mut self) -> Result<Token, ParseError>{
        self.lexer_next();
        try!(self.expect_token(&[Token::Whitespace]));
        self.lexer_next();
        return Ok(Token::Whitespace);
    }
}



#[derive(Debug)]
pub enum ParseError {
    //general errors
    UnknownError,
    EmptyQueryError,
    //End of file, used internal
    EofError,
    TestError,
    ToDo,

    // Syntax errors:
    WrongKeyword(Span),
    WrongToken(Span),
    NotAKeyword(Span),
    NotAToken(Span),
    NotAWord(Span),




    //Used for debugging
    DebugError(String)
// TODO: introduce good errors and think more about it
}
