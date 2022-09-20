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

use std::collections::HashMap;
use db::lib::{db_connect, QuestionQueryResult, query_questions, QuestionOptions};
use init::lib::get_lc_questions_on_init;
use serde::Serialize;



#[derive(Serialize)]
struct CommandResult {
    message: String,
    status: i32
}


#[derive(Serialize)]
struct QuestionCommandResult {
    data: HashMap<i32, QuestionQueryResult>,
    result: CommandResult,
}


async fn preload_lc_questions_into_db() -> CommandResult {
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

    CommandResult { message, status }
}


#[tauri::command]
#[allow(dead_code)]
// rustc thinks this is dead, but its not. will be invoked as command from FE
async fn get_questions(options: QuestionOptions) -> QuestionCommandResult {
    let questions_query_result = query_questions(options).await;
    let mut message: String = "question query successful".to_string();
    let mut status: i32 = 500;
    let result_map: HashMap<i32, QuestionQueryResult>;
    match questions_query_result {
        Ok(q_map) => {
            result_map = q_map;
        }
        Err(err) => {
            println!("could not load questions from db: {:?}", err);
            result_map = HashMap::new();
            message = "question query failed".to_string();
            status = 500;
        }
    }

    QuestionCommandResult {
        data: result_map,
        result: CommandResult { message, status }
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = db_connect();
    diesel_migrations::run_pending_migrations(&conn).expect("migration error");
    preload_lc_questions_into_db().await;

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_questions])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    Ok(())
}
