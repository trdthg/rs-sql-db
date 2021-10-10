// ????  SQL转TokenStream, TokenStream解析, sql方法映射, B+树, 文件存储
// TODO  网络接口, 建表

// ????  与B+树分离
// TODO  根页分裂的处理, 删除
#![allow(dead_code, unused_variables)]

use clap::{App, Arg, SubCommand};

fn main() {
    // start a socket server
    let matches = App::new("mydb")
        .author("trdthg")
        .version("0.0.1")
        .about("this is a micro database for me to study rust and mysql")
        .arg(
            Arg::with_name("start_server")
                .short("s")
                .long("server")
                .help("start the database server"),
        )
        .subcommands(vec![
            SubCommand::with_name("client"),
            SubCommand::with_name("server"),
        ])
        .get_matches();
    match matches.subcommand_name() {
        Some("server") => {
            println!("server started ...");
        }
        Some("client") => {
            println!("client started ...");
        }
        _ => {
            println!("inlegal command!")
        }
    }
}
