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


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = db_connect();
    diesel_migrations::run_pending_migrations(&conn).expect("migration error");

    let q_check = get_lc_questions_on_init(&conn).await;
    match q_check {
        Ok(_) => {}
        Err(err) => println!("error preloading leetcode questions: {:?}", err)
    }
    

    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    Ok(())
}
