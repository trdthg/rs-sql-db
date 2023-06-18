pub mod core;
pub mod http;
pub mod parser;
pub mod bptree;

#[cfg(test)]
mod test {

    use crate::{
        core::{page::PageManager, row::RowManager},
        parser::{parser::Parser, token},
    };

    #[test]
    fn a() {
        let sql = "create table user (
            id int,
          col2 int ,
       col3 char(5) ,
    col4 varchar(11) ,
name varchar(15) not null
)";
        // let sql = "   \ninsert into user(id,col2,col3,col4,name)values(1,4,aaaaa,bbbb, cc)where id=1; ";
        let token_stream = token::trim_to_token_stream(&token::trim_code(sql));
        println!("{:#?}", token_stream);
        let mut parser: Parser = Parser::new();
        parser.parse(token_stream).execute();
        println!("{:#?}", parser);
        // let f = OpenOptions::new().read(true).open("student.db").unwrap();
        // let mut buf = [0; 8];
        // for i in 0..7 {
        //     f.read_at(&mut buf, 0 + i * 8);
        //     println!("{}", usize::from_ne_bytes(buf));
        // }

        let sql =
            "   \ninsert into user(id,col2,col3,col4,name)values(1,4,aaaaa,bbbb, cc)where id=1; ";
        let token_stream = token::trim_to_token_stream(&token::trim_code(sql));
        let mut parser = Parser::new();
        parser.parse(token_stream);
        println!("{:?}", parser);
        let mut rowmanager = RowManager::new("user.frm");
        let bytes = rowmanager.from_parser(parser);
        println!("{:?}", bytes);

        let mut pagemanager = PageManager::read_file("user.db");
        pagemanager.insert(1, bytes);
        let res = pagemanager.select_recursive(0, 1);
        let res = rowmanager.to_row(res.unwrap());
        println!("{:?}", res);
    }

    #[test]
    fn b() {}
}
