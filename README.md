# my-rust-db

## 简介

本项目是在刚看完 [The Book](https://rustwiki.org/zh-CN/book/) 和
[Learn Rust With Entirely Too Many Linked Lists](https://rust-unofficial.github.io/too-many-lists/)
后不久的练手项目。难得莅临的各位大佬看个乐就行了。

## 功能介绍

### SQL Parser

实现了一个简易的 SQL 语法解析器，目前支持的语句有：

- insert into
- delete from
- update
- select from where
- create table xxx ()

建表支持的只有 int、char(n) 和 varchar 3 种类型

基本流程是 sql -> format -> token_stream -> struct

### Page

_page.rs_ 中包含的主要结构体是 `PageManager`，实现了一个简单的基于 B+ 树的数据库文件管理器。只实现了 B+
树的插入和查询操作，删除对应的 B+ 树操作更为复杂，这里并没有实现。

> tree 文件夹则包含了一个纯内存 B+ 树的插入

当插入数据足够单页大小时(这里设置的是 512 字节)，单个页会进行分裂操作，生成一个父页和两个子页。

### Row

_row.rs_ 的结构和 InnoDB 描述的相对更为简单，省区了回滚指针等部分。RowManager 会将单行数据解析为 `Rust` 数据类型。

> http 文件夹则是对 python http 标准库的复刻，事实证明，无脑复刻并不行，在 Rust 里，还是要按照 Rust 的风格实现。

### 参考效果

创建数据表示例

```rs
let sql = "create table user (
            id int,
          col2 int ,
       col3 char(5) ,
    col4 varchar(11) ,
name varchar(15) not null
)";
// 转化为 token_stream
let token_stream = token::trim_to_token_stream(&token::trim_code(sql));
let mut parser: Parser = Parser::new();
// 解析语句，并创建对应的 frm 文件
parser.parse(token_stream).execute();
```

插入示例

```rs
let sql =
    "   \ninsert into user(id,col2,col3,col4,name)values(1,4,aaaaa,bbbb, cc)where id=1; ";
let token_stream = token::trim_to_token_stream(&token::trim_code(sql));
let mut parser = Parser::new();
parser.parse(token_stream);
// 加载表结构文件
let mut rowmanager = RowManager::new("user.frm");
let bytes = rowmanager.from_parser(parser);
// 加载数据文件
let mut pagemanager = PageManager::read_file("user.db");
// 插入
pagemanager.insert(1, bytes);
```

查询示例

```rs
let data = pagemanager.select_recursive(0, 1).unwrap();
let res = rowmanager.to_row(data);
println!("{:?}", res);
```
