use std::collections::HashMap;
use std::{fmt::Result, iter};

use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug, Clone)]
pub enum TokenState {
    START,    // 需要判断后方数据
    ANNO_PRE, // /
    ANNO_A,   // //
    ANNO_B,   // /*
    ANNO_B2,  // /* * / *
    READY,    // 注释结束了, 且后面不是注释
    BLANK,
}

pub fn remove_annotation(code: &str) -> String {
    let mut state: TokenState = TokenState::START;
    let mut old_state: TokenState = TokenState::START;
    let mut mark: usize = 0;
    let mut chars: Vec<char> = code.chars().into_iter().collect();
    let mut chars_: Vec<char> = chars.clone();

    let res: String = chars
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            match c {
                '/' => match state {
                    TokenState::START => {
                        if let Some(&c_1) = chars_.get(i + 1) {
                            if c_1 == '/' || c_1 == '*' {
                                state = TokenState::ANNO_PRE; // 可能是/ 或注释
                            } else {
                                state = TokenState::READY;
                            }
                        }
                    }
                    TokenState::READY => {
                        if let Some(&c_1) = chars_.get(i + 1) {
                            if c_1 == '/' || c_1 == '*' {
                                state = TokenState::ANNO_PRE; // 可能是/ 或注释
                            }
                        }
                    }
                    TokenState::ANNO_PRE => {
                        state = TokenState::ANNO_A; // 第一种注释
                    }
                    TokenState::ANNO_B2 => state = TokenState::START,
                    _ => {}
                },
                '\n' => match state {
                    TokenState::ANNO_A => {
                        state = TokenState::START;
                    }
                    _ => {
                        state = TokenState::READY;
                    }
                },
                '*' => match state {
                    TokenState::ANNO_PRE => {
                        state = TokenState::ANNO_B;
                    }
                    TokenState::ANNO_B => {
                        state = TokenState::ANNO_B2;
                    }
                    _ => {
                        state = TokenState::READY;
                    }
                },
                ' ' => match state {
                    TokenState::READY => {
                        if let Some(&c_1) = chars_.get(i + 1) {
                            if c_1 == ' ' {
                                old_state = state.clone();
                                state = TokenState::BLANK; // 可能是/ 或注释
                            }
                        }
                    }
                    TokenState::BLANK => {
                        if let Some(&c_1) = chars_.get(i + 1) {
                            if c_1 != ' ' {
                                state = old_state.clone();
                            }
                        }
                    }
                    _ => {}
                },
                _ => match state {
                    TokenState::ANNO_A => {}
                    TokenState::ANNO_B => {}
                    TokenState::ANNO_B2 => {}
                    _ => {
                        state = TokenState::READY;
                    }
                },
            };
            let mut tmp;
            match state {
                TokenState::READY => {
                    tmp = Ok(c);
                }
                _ => {
                    tmp = Err("");
                }
            }
            tmp.ok()
        })
        .collect();
    res.as_str().trim().to_string()
}

lazy_static! {
    static ref OPERATIONS: Vec<String> = "+-*/%=&|<>!"
        .chars()
        .into_iter()
        .map(|c| c.to_string())
        .collect();
    static ref BOUNDARYS: Vec<String> = "(),;".chars().into_iter().map(|c| c.to_string()).collect();
    static ref ELEMTYPE: Vec<String> = vec!["int", "char", "date", "varchar", "time"]
        .iter()
        .map(|x| x.to_string())
        .collect();
    static ref KEYWORDS: Vec<String> = vec![
        "select",
        "insert",
        "delete",
        "update",
        "from",
        "create",
        "database",
        "as",
        "drop",
        "alter",
        "into",
        "where",
        "join",
        "set",
        "use",
        "table",
        "index",
        "primary",
        "in",
        "between",
        "like",
        "and",
        "or",
        "key",
        "by",
        "character",
        "decimal",
        "null",
        "not",
        "values",
        "order"
    ]
    .iter()
    .map(|x| x.to_string())
    .collect();
}
pub fn trim_code(code: &str) -> String {
    // 去注释
    let mut re = Regex::new("/\\*[\\s\\S]*\\*/").unwrap();
    let mut code = Regex::replace_all(&re, &code, "");
    re = Regex::new("//.*\n").unwrap();
    let mut code = Regex::replace_all(&re, &code, "");
    // 去空格
    re = Regex::new("\\s+").unwrap();
    let mut code = Regex::replace_all(&re, &code, " ");

    // 格式化运算符和边界符
    re = Regex::new("\\s?=\\s?").unwrap();
    let mut code = Regex::replace_all(&re, &code, " = ");
    re = Regex::new("\\s?\\(\\s?").unwrap();
    let mut code = Regex::replace_all(&re, &code, " ( ");
    re = Regex::new("\\s?\\)\\s?").unwrap();
    let mut code = Regex::replace_all(&re, &code, " ) ");
    re = Regex::new("\\s?,\\s?").unwrap();
    let mut code = Regex::replace_all(&re, &code, " , ");
    re = Regex::new("\\s?;\\s?").unwrap();
    let mut code = Regex::replace_all(&re, &code, " ; ");

    // 变小写
    let code = code.to_ascii_lowercase();
    let code = code.trim();
    code.to_string()
}

#[derive(Debug)]
pub enum TokenType {
    KeyWord,
    String,
    Operation,
    Boundary,
    Number,
    ELEMTYPE,
}
#[derive(Debug)]
pub struct Token {
    pub tokentype: TokenType,
    pub value: String,
    pub offset: usize,
    pub length: usize,
}
impl Token {
    pub fn new(tokentype: TokenType, value: String, offset: usize, length: usize) -> Self {
        Token {
            tokentype,
            value,
            offset,
            length,
        }
    }
}

pub fn trim_to_token_stream(code: &str) -> Vec<Token> {
    let mut vec = Vec::new();
    let words: Vec<&str> = code.split_whitespace().collect();
    let mut str_offset = 0;
    for (i, word) in words.iter().enumerate() {
        let word = word.to_string();
        let word_len = word.len();
        if OPERATIONS.contains(&word) {
            vec.push(Token::new(TokenType::Operation, word, str_offset, word_len))
        } else if BOUNDARYS.contains(&word) {
            vec.push(Token::new(TokenType::Boundary, word, str_offset, word_len))
        } else if KEYWORDS.contains(&word) {
            vec.push(Token::new(TokenType::KeyWord, word, str_offset, word_len))
        } else if ELEMTYPE.contains(&word) {
            vec.push(Token::new(TokenType::ELEMTYPE, word, str_offset, word_len))
        } else {
            vec.push(Token::new(TokenType::String, word, str_offset, word_len))
        }
        str_offset += word_len + 1;
    }
    vec
}

#[test]
fn test() {
    let code = r#"
        SELECT  * from  adwdw where   a   =ad  and b=ad and ; 
        insert into user(id,name)values(1,"saadwdd")where id=1;    
    "#;
    let code = trim_code(code);
    println!("{:?}", code);
    println!("{}", code);
    let token_stream = trim_to_token_stream(&code);
    println!("{:#?}", token_stream);
}
