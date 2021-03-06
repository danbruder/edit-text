#![feature(extern_in_paths)]

#[macro_use] extern crate quicli;
extern crate edit_server;
extern crate serde_json;

use extern::{
    diesel::connection::Connection,
    quicli::prelude::*,
    edit_server::db::*,
};

#[derive(Debug, StructOpt)]
enum Cli {
    #[structopt(name = "list")]
    List {
        #[structopt(long = "source")]
        source: Option<String>,
    },

    #[structopt(name = "clear")]
    Clear,
}

main!(|args: Cli| {
    let db = db_connection();

    match args {
        Cli::Clear => {
            clear_all_logs(&db)?;
            db.execute("VACUUM").unwrap();
            eprintln!("cleared logs.");
        }
        Cli::List { source } => {
            let logs = if let Some(source) = source {
                eprintln!("Filter by source: {}", source);
                select_logs(&db, &source)?
            } else {
                all_logs(&db)?
            };

            eprintln!("Printing {} logs...", logs.len());

            for log in logs {
                println!("{}", serde_json::to_string(&log).unwrap());
            }
        }
    }
});
