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

pub mod models;
pub mod schema;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use dotenv::dotenv;
use models::Question;
use schema::question;
use std::env;

pub fn db_connect() -> SqliteConnection {
    dotenv().ok();

    let db_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL not set in .env");
    SqliteConnection::establish(&db_url)
        .expect(&format!("Error connecting to {}", db_url))
}

fn main() {
    let conn = db_connect();
    diesel_migrations::run_pending_migrations(&conn).expect("migration error");

    let test = question::dsl::question.load::<Question>(&conn)
        .expect("error loading test from db");
    println!("loaded {} questions", test.len());

    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
