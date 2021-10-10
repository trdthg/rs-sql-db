use std::{collections::HashMap, convert::TryInto};

use bit::BitIndex;

use crate::parse::parser::{Field, Parser};

// create table demo (
//     col3 int ,
//     col2 char(5) ,
//     col1 varchar(11) ,
//     col4 varchar(15) not null
// )
struct Row {
    empty_list_offset: u8,
    variablelist: Vec<u8>,
    emptylist: u8,
    datalist: Vec<u8>,
}
impl Default for Row {
    fn default() -> Self {
        Self {
            variablelist: Default::default(),
            emptylist: Default::default(),
            datalist: Default::default(),
            empty_list_offset: Default::default(),
        }
    }
}
impl Row {
    pub fn new() -> Self {
        Row::default()
    }
}
// 功能是把文件中的记录解析为需要的类型, 或者是能把新插入的数据按照类型重新编码为最终会被插入到文件里的字节流
pub struct RowManager {
    fields: Vec<Field>,
}
impl RowManager {
    pub fn new(frm_name: &str) -> Self {
        //
        let a = Parser::de(frm_name);
        let fields = a.Ptable;
        println!("self.map: {:#?}", &fields);
        Self { fields }
    }

    pub fn from_parser(&mut self, parser: Parser) -> Vec<u8> {
        let mut row = Row::new();
        let mut empty_list_len = 0;
        for (i, (k, v)) in parser.Pfields.iter().zip(parser.Pvalues).enumerate() {
            let field = &self.fields[i];
            // 如果字段属于变长字段类型就要在变长字段列表里标记长度,
            // 如果变长字段为空

            //   变长字段 --> 标记长度
            //      |
            //      | 字符串为空
            //      v
            //   标记为空

            //   普通字段 --> 不标记长度
            //       |
            //       | 字符串为空
            //       v
            //    标记为空
            if v.len() == 0 {
                row.emptylist.set_bit(i, true);
            } else {
                println!("{}", field.fieldtype.as_str());
                match field.fieldtype.as_str() {
                    "int" => {
                        let num: u64 = v.parse().unwrap();
                        println!("num: {} ", &num);
                        let b = num.to_ne_bytes();
                        row.datalist.append(&mut b.to_vec());
                    }
                    "varchar" => {
                        // 添加变长字段的长度
                        empty_list_len += 1;
                        row.variablelist.push(v.as_bytes().len() as u8);
                        row.datalist.append(&mut v.as_bytes().to_vec());
                        println!("v: {} {}", &v, v.as_bytes().len() as u8);
                    }
                    _ => {
                        // 添加普通字段
                        row.datalist.append(&mut v.as_bytes().to_vec());
                        println!("v: {} {}", &v, v.as_bytes().len() as u8);
                    }
                }
            }
        }
        row.empty_list_offset = row.variablelist.len() as u8 + 1;

        let mut res: Vec<u8> = vec![row.empty_list_offset];
        println!("{:?}", res);
        res.append(&mut row.variablelist);
        println!("{:?}", res);
        res.push(row.emptylist);
        println!("{:?}", res);
        res.append(&mut row.datalist);
        res
    }

    pub fn to_row(&mut self, data: Vec<u8>) -> HashMap<String, Option<String>> {
        let mut sbuf: Vec<u8> = vec![];
        let empty_list_offset = data[0];
        // 空值列表
        let emptylist: u8 = data[empty_list_offset as usize].bit_range(0..self.fields.len());
        // 变长字段列表
        let mut variablelist = vec![];
        for i in 1..empty_list_offset {
            let length: u8 = data[i as usize];
            variablelist.push(length);
        }
        println!("empty_list.len(): {}", self.fields.len());
        println!("emptylist: {:#b}", emptylist);
        println!("variable_list: {:?}", variablelist);
        let datalist = data[(empty_list_offset + 1) as usize..].to_vec();
        println!("datalist: {:?} datalist_len: {}", datalist, datalist.len());
        let mut res: HashMap<String, Option<String>> = HashMap::new();
        let mut data_start_offset = 0;
        for (i, field) in self.fields.iter().enumerate() {
            println!(
                "============================field.value: {}, field.fieldtype: {}",
                field.value, field.fieldtype
            );
            if emptylist.bit(i) == true {
                res.insert(field.value.clone(), None);
                continue;
            } else {
                // 若不为空, 获取到bitsize查询到字段并转为字符串
                print!("start_offset: {} ", data_start_offset);
                let (bitlen, data): (u8, String) = match field.fieldtype.as_str() {
                    "int" => {
                        let bitlen: u8 = 8;
                        let mut buf: [u8; 8] = [0; 8];
                        for (i, j) in
                            (data_start_offset..(data_start_offset + bitlen as usize)).enumerate()
                        {
                            buf[i] = datalist[j].clone();
                        }

                        let data = usize::from_ne_bytes(buf).to_string();
                        print!("data: {} ", data);
                        (bitlen, data)
                    }
                    "varchar" => {
                        let bitlen = variablelist.remove(0);
                        let data: Vec<u8> = datalist
                            [data_start_offset..(data_start_offset + bitlen as usize)]
                            .to_vec();
                        print!("data: {:?} ", data);
                        let data = String::from_utf8_lossy(&data);
                        print!("data: {:?} ", data);
                        (bitlen, data.to_string())
                    }
                    _ => {
                        let bitlen = field.bitsize as u8;
                        let data: Vec<u8> = datalist[data_start_offset as usize
                            ..(data_start_offset + bitlen as usize) as usize]
                            .to_vec();
                        let data = String::from_utf8_lossy(&data).to_string();
                        (bitlen, data)
                    }
                };
                print!("bit_size: {} ", &bitlen);
                data_start_offset += bitlen as usize;
                println!("{} {} {}", field.value, bitlen, data);
                res.insert(field.value.clone(), Some(data.to_string()));
            }
        }
        res
    }
}

#[cfg(test)]
mod test {
    use super::super::super::parse::parser::Parser;
    use super::super::super::parse::token;
    use super::RowManager;

    #[test]
    fn b() {
        let sql =
            "   \ninsert into user(id,col2,col3,col4,name)values(1,4,aaaaa,bbbb, cc)where id=1; ";
        let mut token_stream = token::trim_to_token_stream(&token::trim_code(sql));
        let mut parser: Parser = Parser::new();
        parser.parse(token_stream);
        println!("{:?}", parser);

        let mut rowmanager: RowManager = RowManager::new("user.frm");
        let bytes = rowmanager.from_parser(parser);
        println!("{:?}", bytes);
        let res = rowmanager.to_row(bytes);
        println!("{:#?}", res);
    }

    #[test]
    fn a2() {
        let a = [1, 2, 3];
        let b = [1, 2, 3];
        for (x, y) in a.iter().zip(b.iter()) {
            //
        }
    }

    #[test]
    fn a() {
        let a: i64 = 0b00000110;
        println!("{}", a);
        let b = a.to_ne_bytes();
        println!("{:?}", b);

        let a = 1;

        let mut value = 0b11010110u8;

        use bit::BitIndex;
        // 8
        println!("{}", u8::bit_length());

        // true
        println!("{}", value.bit(1));

        // 0b10
        println!("{:#b}", value.bit_range(0..2));

        value
            .set_bit(3, true)
            .set_bit(2, false)
            .set_bit_range(5..8, 0b001);

        // 0b111010
        println!("{:#b}", value);
        let mut value = 0b11010110u8;

        // 8
        println!("{}", u8::bit_length());

        // true
        println!("{}", value.bit(1));

        // 0b10
        println!("{:#b}", value.bit_range(0..2));

        value
            .set_bit(3, true)
            .set_bit(2, false)
            .set_bit_range(5..8, 0b001);

        // 0b111010
        println!("{:#b}", value);
        println!("{}", value.bit(0));
        println!("{:#b}", value.bit_range(0..4));
    }
}
