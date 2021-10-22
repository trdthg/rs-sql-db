use crate::parse::token;
use crate::parse::parser;
use crate::core::pagemanager;
use crate::core::rowmanager;

pub struct DBManager {
    parser: parser::Parser,
    rowmanager: rowmanager::RowManager,
    pagemanager: pagemanager::PageManager,

}
impl DBManager {
    pub fn new(scheme: &str) -> Self {
        let mut parser: parser::Parser = parser::Parser::new();
        let mut rowmanager: rowmanager::RowManager = rowmanager::RowManager::new("user.frm");
        let mut pagemanager = pagemanager::PageManager::read_file("sss.db");

        DBManager{
            parser, rowmanager, pagemanager
        }
    }
    pub fn execute(&mut self, sql: &str) {
        // 获取sql

        // 解析为tokenstream
        let mut token_stream = token::trim_to_token_stream(&token::trim_code(sql));
        // 解析出执行的sql方法
        self.parser.parse(token_stream);

        let bytes = self.rowmanager.from_parser(self.parser);

        self.pagemanager.insert(1, bytes);
        let res = self.pagemanager.select_recursive(0, 1);
        let res = self.rowmanager.to_row(res.unwrap());
        println!("{:?}", res);
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn execute() {
        let sql =
        "   \ninsert into user(id,col2,col3,col4,name)values(1,4,aaaaa,bbbb, cc)where id=1; ";
        let mut dbmanager: DBManager = DBManager::new("user");
        dbmanager.execute(sql);
    }

    #[test]
    fn a() {
        let sql =
            "   \ninsert into user(id,col2,col3,col4,name)values(1,4,aaaaa,bbbb, cc)where id=1; ";
        let mut token_stream = token::trim_to_token_stream(&token::trim_code(sql));
        let mut parser: parser::Parser = parser::Parser::new();
        parser.parse(token_stream);
        println!("{:?}", parser);
        let mut rowmanager: rowmanager::RowManager = rowmanager::RowManager::new("assets/user.frm");
        let bytes = rowmanager.from_parser(parser);
        println!("{:?}", bytes);
        let mut pagemanager = pagemanager::PageManager::read_file("assets/sss.db");
        pagemanager.insert(1, bytes);
        let res = pagemanager.select_recursive(0, 1);
        let res = rowmanager.to_row(res.unwrap());
        println!("{:?}", res);
    }
}
