use directories::BaseDirs;
use memoake_lib::db;

use std::{
    env::args,
    io::{self, Read},
    path::PathBuf,
};

fn get_db_path() -> PathBuf {
    let base_dirs = BaseDirs::new().expect("failed to get home directory");

    base_dirs
        .data_dir()
        .join("com.memoake.app")
        .join("memoake.db")
}

fn main() {
    let db_path = get_db_path();
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: memoake-cli <command> <content>");
    }

    let command = &args[1];
    // let content = &args[2];

    let conn = db::connect_db_at_path(&db_path).expect("failed to connect db");

    match command.as_str() {
        "list" => {
            let memos = db::get_all_memo(conn).unwrap();
            for m in memos {
                println!("[{}] {}", m.id, m.content);
            }
        }
        "list-json" => {
            let memos = db::get_all_memo(conn).unwrap();
            println!("{}", serde_json::to_string(&memos).unwrap());
        }
        "create" => {
            if args.len() < 3 {
                println!("Usage: memoake-cli create <content>");
                return;
            }
            let content = &args[2];
            let memo = db::create_memo(conn, content).unwrap();

            println!("success to create memo, id: {}", memo.id);
        }
        _ => println!("Unknown command"),
    }
}
