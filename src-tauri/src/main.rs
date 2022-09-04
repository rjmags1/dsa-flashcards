#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
embed_migrations!("./migrations");
extern crate dotenv;

mod db;
mod network;
mod init;

use db::lib::db_connect;
use init::lib::get_lc_questions_on_init;
use serde::Serialize;



#[derive(Serialize)]
struct CommandResult {
    message: String,
    status: i32
}


#[tauri::command]
#[allow(dead_code)] 
// rustc thinks this is dead, but its not. will be invoked as command from FE
async fn preload_lc_questions_into_db() -> Result<CommandResult, CommandResult> {
    let conn = db_connect();
    let q_check = get_lc_questions_on_init(&conn).await;
    let mut message: String = "Successfully preloaded leetcode questions".to_string();
    let mut status: i32 = 200;
    match q_check {
        Ok(_) => {}
        Err(err) => {
            println!("leetcode question preload failed, {:?}", err);
            message = "failed to preload leetcode questions".to_string();
            status = 500;
        }
    }

    Ok(CommandResult { message, status })
}




#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = db_connect();
    diesel_migrations::run_pending_migrations(&conn).expect("migration error");

    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    Ok(())
}
