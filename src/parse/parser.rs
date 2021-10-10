use super::token::{self, Token, TokenState, TokenType};
use std::io::{BufRead, BufReader, Read, Write};
use std::{collections::HashMap, convert::TryInto, fs::File};

use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Copy)]
pub enum ParseState {
    Begin,
    InSelect,
    InFrom,
    InWhere,
    InInsert,
    InInto,
    InValues,
    InUpdate,
    InDelete,
    InSet,
    InOrder,
    InBoundary,
    End,

    InCreate,
    InTable,
    InFields,
    InField,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Ope {
    key: Option<String>,
    operation: Option<String>,
    value: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Field {
    pub value: String,
    pub fieldtype: String,
    pub bitsize: usize,
    pub can_null: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Parser {
    pub method: String,
    pub table: String,
    pub Pfields: Vec<String>,
    pub Pvalues: Vec<String>,
    pub Pwhere: Vec<Ope>,
    pub Ptable: Vec<Field>,
}

impl Default for Parser {
    fn default() -> Self {
        Self {
            method: Default::default(),
            table: Default::default(),
            Pfields: Default::default(),
            Pvalues: Default::default(),
            Pwhere: Default::default(),
            Ptable: Default::default(),
        }
    }
}
impl Default for Ope {
    fn default() -> Self {
        Self {
            key: Default::default(),
            operation: Default::default(),
            value: Default::default(),
        }
    }
}
impl Parser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse(&mut self, token_stream: Vec<Token>) -> &mut Self {
        match &token_stream[0].tokentype {
            TokenType::KeyWord => match token_stream[0].value.as_str() {
                "select" => {
                    return Self::select(self, token_stream);
                }
                "insert" => {
                    return Self::insert(self, token_stream);
                }
                "update" => {
                    return Self::update(self, token_stream);
                }
                "delete" => {
                    return Self::delete(self, token_stream);
                }
                "create" => {
                    return Self::create_table(self, token_stream);
                }
                _ => {}
            },
            _ => {}
        }
        self
    }

    pub fn create_table(&mut self, token_stream: Vec<Token>) -> &mut Self {
        let mut parsestate = ParseState::Begin;

        for (i, token) in token_stream.iter().enumerate() {
            let value = token.value.as_str();
            let tokentype = &token.tokentype;
            match tokentype {
                TokenType::KeyWord => match value {
                    "create" => {
                        self.method = "create".to_string();
                        parsestate = ParseState::InCreate;
                    }
                    "table" => {
                        parsestate = ParseState::InTable;
                    }
                    "null" => {
                        self.Ptable.last_mut().unwrap().can_null = false;
                    }
                    _ => {}
                },
                TokenType::String => match parsestate {
                    ParseState::InTable => {
                        self.table = value.to_string();
                    }
                    ParseState::InFields => {
                        self.Ptable.push(Field {
                            value: value.to_string(),
                            fieldtype: String::new(),
                            bitsize: 0,
                            can_null: true,
                        });
                    }
                    ParseState::InField => {
                        println!("{}", value);
                        self.Ptable.last_mut().unwrap().bitsize = value.parse().unwrap();
                    }
                    _ => {}
                },
                &TokenType::ELEMTYPE => match value {
                    "int" => {
                        self.Ptable.last_mut().unwrap().fieldtype = "int".to_string();
                        self.Ptable.last_mut().unwrap().bitsize = 8;
                    }
                    "varchar" => {
                        self.Ptable.last_mut().unwrap().fieldtype = "varchar".to_string();
                    }
                    "char" => {
                        self.Ptable.last_mut().unwrap().fieldtype = "char".to_string();
                    }
                    _ => {}
                },
                &TokenType::Boundary => match parsestate {
                    ParseState::InTable => match value {
                        "(" => {
                            parsestate = ParseState::InFields;
                        }
                        _ => {}
                    },
                    ParseState::InFields => match value {
                        "(" => {
                            parsestate = ParseState::InField;
                        }
                        "," => {
                            parsestate = ParseState::InFields;
                        }
                        _ => {}
                    },
                    ParseState::InField => match value {
                        ")" => {
                            parsestate = ParseState::InFields;
                        }
                        _ => {}
                    },
                    _ => {}
                },

                _ => {}
            }
        }

        self
    }

    pub fn select(&mut self, token_stream: Vec<Token>) -> &mut Self {
        let mut parsestate = ParseState::Begin;
        for (i, token) in token_stream.iter().enumerate() {
            let value = token.value.as_str();
            let tokentype = &token.tokentype;
            match tokentype {
                TokenType::KeyWord => match value {
                    "select" => {
                        self.method = "select".to_string();
                        parsestate = ParseState::InSelect;
                    }
                    "from" => {
                        parsestate = ParseState::InFrom;
                    }
                    "where" => {
                        parsestate = ParseState::InWhere;
                    }
                    _ => {}
                },
                TokenType::String => match parsestate {
                    ParseState::InFrom => {
                        self.table = value.to_string();
                    }
                    ParseState::InSelect => {
                        self.Pfields.push(value.to_string());
                    }

                    _ => {}
                },
                TokenType::Operation => match value {
                    "*" => match parsestate {
                        ParseState::InSelect => {
                            // push all
                        }
                        _ => {}
                    },
                    "=" => match parsestate {
                        ParseState::InWhere => {
                            if let (Some(k), Some(v)) =
                                (token_stream.get(i - 1), token_stream.get(i + 1))
                            {
                                self.Pwhere.push(Ope {
                                    operation: Some("=".to_string()),
                                    key: Some(k.value.clone()),
                                    value: Some(v.value.clone()),
                                });
                            }
                        }

                        _ => {}
                    },
                    _ => {}
                },
                _ => {}
            }
        }
        self
    }

    pub fn insert(&mut self, token_stream: Vec<Token>) -> &mut Self {
        let mut parsestate = ParseState::Begin;
        for (i, token) in token_stream.iter().enumerate() {
            let value = token.value.as_str();
            let tokentype = &token.tokentype;
            match tokentype {
                TokenType::KeyWord => match value {
                    "insert" => {
                        self.method = "insert".to_string();
                        parsestate = ParseState::InInsert;
                    }
                    "values" => {
                        parsestate = ParseState::InValues;
                    }
                    _ => {}
                },
                TokenType::String => match parsestate {
                    ParseState::InInsert => {
                        self.table = value.to_string();
                    }
                    _ => {}
                },
                TokenType::Boundary => match value {
                    "(" => match parsestate {
                        ParseState::InInsert => {
                            parsestate = ParseState::InInto;
                            let mut mark = i;
                            loop {
                                if let Some(t) = token_stream.get(mark) {
                                    match t.tokentype {
                                        TokenType::String => {
                                            self.Pfields.push(t.value.to_string());
                                        }
                                        TokenType::Boundary => {
                                            if t.value.as_str() == ")" {
                                                break;
                                            }
                                        }
                                        _ => {}
                                    }
                                    mark += 1;
                                }
                            }
                        }
                        ParseState::InValues => {
                            let mut mark = i;
                            loop {
                                if let Some(t) = token_stream.get(mark) {
                                    match t.tokentype {
                                        TokenType::String => {
                                            self.Pvalues.push(t.value.to_string());
                                        }
                                        TokenType::Boundary => {
                                            if t.value.as_str() == ")" {
                                                break;
                                            }
                                        }
                                        _ => {}
                                    }
                                    mark += 1;
                                }
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                },
                TokenType::Number => {}
                _ => {}
            }
        }

        self
    }

    pub fn update(&mut self, token_stream: Vec<Token>) -> &mut Self {
        let mut parsestate = ParseState::Begin;
        for (i, token) in token_stream.iter().enumerate() {
            let value = token.value.as_str();
            let tokentype = &token.tokentype;
            match tokentype {
                TokenType::KeyWord => match value {
                    "update" => {
                        self.method = "update".to_string();
                        parsestate = ParseState::InUpdate;
                    }
                    "where" => {
                        parsestate = ParseState::InWhere;
                    }
                    "set" => {
                        parsestate = ParseState::InSet;
                    }
                    _ => {}
                },
                TokenType::String => match parsestate {
                    ParseState::InUpdate => {
                        self.table = value.to_string();
                    }
                    _ => {}
                },
                TokenType::Operation => match value {
                    "=" => match parsestate {
                        ParseState::InSet => match tokentype {
                            TokenType::Operation => {
                                if value == "=" {
                                    if let (Some(k), Some(v)) =
                                        (token_stream.get(i - 1), token_stream.get(i + 1))
                                    {
                                        self.Pfields.push(k.value.to_string());
                                        self.Pvalues.push(v.value.to_string());
                                    }
                                }
                            }
                            _ => {}
                        },
                        ParseState::InWhere => {
                            if let (Some(k), Some(v)) =
                                (token_stream.get(i - 1), token_stream.get(i + 1))
                            {
                                self.Pwhere.push(Ope {
                                    operation: Some("=".to_string()),
                                    key: Some(k.value.clone()),
                                    value: Some(v.value.clone()),
                                });
                            }
                        }
                        _ => {}
                    },

                    _ => {}
                },
                TokenType::Number => {}
                _ => {}
            }
        }
        self
    }

    pub fn delete(&mut self, token_stream: Vec<Token>) -> &mut Self {
        let mut parsestate = ParseState::Begin;
        for (i, token) in token_stream.iter().enumerate() {
            let value = token.value.as_str();
            let tokentype = &token.tokentype;
            match tokentype {
                TokenType::KeyWord => match value {
                    "delete" => {
                        self.method = "delete".to_string();
                        parsestate = ParseState::InDelete;
                    }
                    "from" => {
                        parsestate = ParseState::InFrom;
                    }
                    "where" => {
                        parsestate = ParseState::InWhere;
                    }
                    _ => {}
                },
                TokenType::String => match parsestate {
                    ParseState::InFrom => {
                        self.table = value.to_string();
                    }
                    _ => {}
                },
                TokenType::Operation => match value {
                    "*" => match parsestate {
                        ParseState::InSelect => {}
                        _ => {}
                    },
                    "=" => match parsestate {
                        ParseState::InWhere => match tokentype {
                            TokenType::Operation => {
                                if value == "=" {
                                    if let (Some(k), Some(v)) =
                                        (token_stream.get(i - 1), token_stream.get(i + 1))
                                    {
                                        self.Pwhere.push(Ope {
                                            operation: Some("=".to_string()),
                                            key: Some(k.value.clone()),
                                            value: Some(v.value.clone()),
                                        });
                                    }
                                }
                            }
                            _ => {}
                        },
                        _ => {}
                    },

                    _ => {}
                },
                TokenType::Number => {}
                _ => {}
            }
        }

        self
    }

    pub fn execute(&mut self) {
        match self.method.as_str() {
            "insert" => {

                //
            }
            "create" => {
                Self::se(self, format!("{}.frm", self.table).as_str());
                //
            }
            _ => {}
        }
    }

    pub fn de(name: &str) -> Self {
        let f = File::open(name).unwrap();
        let mut s = String::new();
        let mut reader = BufReader::new(f);
        reader.read_to_string(&mut s).unwrap();
        serde_json::from_str::<Self>(&s).unwrap()
    }

    pub fn se(&self, name: &str) {
        let sered = serde_json::to_string(self).unwrap();
        let mut f = File::create(name).unwrap();
        f.write(sered.as_bytes()).unwrap();
    }
}

#[test]
fn test() {
    let sql = "SELECT id, name from  adwdw where   a   =ad  and b=ad ";
    let sql = "   \ninsert into user(id,name)values(1,\"saadwdd\")where id=1; ";
    let sql = "delete from user where id = 12";
    let sql = "update  user  set          id=1,name=\"acbeix\" where xxx=\"debsxnk\"";
    let sql = "create table user (
                                 id int,
                               col2 int ,
                            col3 char(5) ,
                         col4 varchar(11) ,
                   name varchar(15) not null
   )";
    let sql = "   \ninsert into user(id,col2,col3,col4,name)values(1,4,aaaaa,bbbb, cc)where id=1; ";
    let mut token_stream = token::trim_to_token_stream(&token::trim_code(sql));
    println!("{:#?}", token_stream);
    let mut parser: Parser = Parser::new();
    parser.parse(token_stream).execute();
    println!("{:#?}", parser);
}

#[test]
fn get_fun() {
    let sql = "SELECT  * from  adwdw where   a   =ad  and b=ad ";
    let sql = "   \ninsert into user(id,name)values(1,\"saadwdd\")where id=1; ";
    let sql = "update  user  set          id=1,name=\"acbeix\" where xxx=\"debsxnk\"";
    let sql = "delete from user where id = 12";
    let sql = "   \ninsert into user(id,name)values(1,\"saadwdd\") where id = 1 ";
    let mut token_stream = token::trim_to_token_stream(&token::trim_code(sql));
    // let mut parser: &Parser = Parser::new().parse(token_stream);
    // println!("{:#?}", parser);

    // parser.execute();
}
