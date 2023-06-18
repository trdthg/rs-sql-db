# my-rust-db

## 简介

本项目是在刚看完 [The Book](https://rustwiki.org/zh-CN/book/) 和
[Learn Rust With Entirely Too Many Linked Lists](https://rust-unofficial.github.io/too-many-lists/)
后不久的练手项目，编写于 2021-10。

## 功能介绍

### SQL Parser

实现了一个简易的 SQL 语法解析器，支持的语句：

- insert into
- delete from
- update
- select from where
- create table xxx ()

建表支持的字段类型 int、char(n)、varchar

基本流程是 sql -> format -> token_stream -> struct

### Page

_page.rs_ 中包含的主要结构体是 `PageManager`，实现了一个简单的基于 B+ 树的数据库文件管理。实现了 B+
树的插入和查询操作，删除操作类似于插入，需要多考虑两种情况，这里没有实现。

当插入数据足够单页大小时 (这里设置的是 512 字节)，单个页会进行分裂操作，生成一个父页和两个子页。

> bptree 文件夹则包含了一个纯内存 B+ 树的插入算法

### Row

_row.rs_ 的结构和 InnoDB 描述的相对更为简单，省区了回滚指针等部分。RowManager 会将单行数据解析为 `Rust` 数据类型。

> http 文件夹则是对 python http 标准库的复刻

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
