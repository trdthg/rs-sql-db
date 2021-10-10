pub mod pagemanager;
pub mod rowmanager;
use crate::parse::parser::Parser;
use crate::parse::token;
use pagemanager::PageManager;
use rowmanager::RowManager;

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn a() {
        let sql =
            "   \ninsert into user(id,col2,col3,col4,name)values(1,4,aaaaa,bbbb, cc)where id=1; ";
        let mut token_stream = token::trim_to_token_stream(&token::trim_code(sql));
        let mut parser: Parser = Parser::new();
        parser.parse(token_stream);
        println!("{:?}", parser);
        let mut rowmanager: RowManager = RowManager::new("user.frm");
        let bytes = rowmanager.from_parser(parser);
        println!("{:?}", bytes);
        let mut pagemanager = PageManager::read_file("sss.db");
        pagemanager.insert(1, bytes);
        let res = pagemanager.select_recursive(0, 1);
        let res = rowmanager.to_row(res.unwrap());
        println!("{:?}", res);
    }
}
