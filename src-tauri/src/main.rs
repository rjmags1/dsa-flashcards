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
use schema::{question, question_topic};
use serde::{Serialize, Deserialize};
use serde_json::json;
use std::env;

use crate::models::{NewQuestion, NewQuestionTopic, Question};


pub fn db_connect() -> SqliteConnection {
    dotenv().ok();

    let db_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL not set in .env");
    SqliteConnection::establish(&db_url)
        .expect(&format!("Error connecting to {}", db_url))
}

#[derive(Debug, Serialize, Deserialize)]
struct ResponseTopic {
    name: String,
    id: String,
    slug: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct ResponseQuestion {
    difficulty: String,
    questionId: String,
    title: String,
    titleSlug: String,
    topicTags: Vec<ResponseTopic>,
}

#[derive(Debug, Serialize, Deserialize)]
struct QuestionList {
    questions: Vec<ResponseQuestion>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProblemsetQuestionList {
    problemsetQuestionList: QuestionList,
}

#[derive(Debug, Serialize, Deserialize)]
struct QuestionListResponse {
    data: ProblemsetQuestionList,
}

async fn fetch_all_questions() -> Result<QuestionList, Box<dyn std::error::Error>> {
    let query_string = "query \
        problemsetQuestionList(\
            $categorySlug: String, \
            $limit: Int, \
            $filters: QuestionListFilterInput\
        ) { \
            problemsetQuestionList: questionList( \
                categorySlug: $categorySlug \
                limit: $limit \
                filters: $filters \
            ) { \
                questions: data { \
                    difficulty \
                    questionId \
                    title \
                    titleSlug \
                    topicTags { name id slug }  \
                } \
            }\
        }";
    let req_body = json!({
        "query": query_string,
        "variables": {
            "categorySlug": "",
            "limit": 2400,
            "filters": {},
        }
    });
    let res = reqwest::Client::new()
        .post("https://leetcode.com/graphql")
        .json(&req_body)
        .send()
        .await?;
    let parsed = res.json::<QuestionListResponse>().await?;


    Ok(parsed.data.problemsetQuestionList)

}

fn db_format_question(response_question: &ResponseQuestion) -> Result<NewQuestion, Box<dyn std::error::Error>>  {
    let new_question = NewQuestion {
        question_number: response_question.questionId.parse::<i32>()?,
        title: response_question.title.clone(),
        title_slug: response_question.titleSlug.clone(),
        difficulty: response_question.difficulty.clone(),
        prompt: "".to_string(),
        fetched: true
    };

    Ok(new_question)
}

fn db_format_question_topics(response_question: ResponseQuestion) -> Result<Vec<NewQuestionTopic>, Box<dyn std::error::Error>> {
    let mut question_topics: Vec<NewQuestionTopic> = vec![];
    for topic in response_question.topicTags {
        question_topics.push(NewQuestionTopic { 
            question_number: response_question.questionId.parse::<i32>()?, 
            topic: topic.name.clone(),
        })
    } 

    Ok(question_topics)
}

async fn db_insert_all_questions(conn: SqliteConnection, response_questions: QuestionList) -> Result<(), Box<dyn std::error::Error>> {
    // make vec of NewQuestion from vec of ResponseQuestion
    let mut db_new_questions: Vec<NewQuestion> = vec![];
    let mut db_new_question_topics: Vec<NewQuestionTopic> = vec![];
    for response_question in response_questions.questions {
        db_new_questions.push(db_format_question(&response_question)?);
        let new_topics: Vec<NewQuestionTopic> = db_format_question_topics(response_question)?;
        for topic in new_topics {
            db_new_question_topics.push(topic);
        }
    }

    diesel::insert_into(question::table)
        .values(&db_new_questions)
        .execute(&conn)?;
    diesel::insert_into(question_topic::table)
        .values(&db_new_question_topics)
        .execute(&conn)?;

    Ok(())
}

// fn that checks if db has questions inserted already, and does fetch + insert if not
async fn get_lc_questions(conn: SqliteConnection) -> Result<(), Box<dyn std::error::Error>> {
    let query_string = "SELECT id FROM question WHERE fetched = TRUE LIMIT 1;";
    let q_rows: Vec<Question> =  diesel::sql_query(query_string).load(&conn)?;
    if q_rows.len() == 1 { // there are already 
        return Ok(());
    }

    let fetched_questions = fetch_all_questions().await?;
    db_insert_all_questions(conn, fetched_questions).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = db_connect();
    diesel_migrations::run_pending_migrations(&conn).expect("migration error");

    let q_check = get_lc_questions(conn).await;
    match q_check {
        Ok(_) => {}
        Err(err) => println!("error preloading questions from leetcode: {:?}", err)
    }
    

    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    Ok(())
}
