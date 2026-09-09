use memoake_lib::db;

fn main() {
    let db_path = db::get_default_db_path().expect("failed to get default db path");
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
        "update" => {
            if args.len() < 4 {
                println!("Usage: memoake-cli update <id> <content>");
                return;
            }
            let id: i64 = args[2].parse().expect("failed to parse id");
            let content = &args[3];
            let memo = db::update_memo(conn, id, content).unwrap();

            println!("success to update memo, id: {}", memo.id);
        }
        "delete" => {
            if args.len() < 3 {
                println!("Usage: memoake-cli delete <id>");
                return;
            }
            let id: i64 = args[2].parse().expect("failed to parse id");
            db::delete_memo(conn, id).unwrap();
        }
        _ => println!("Unknown command"),
    }
}
