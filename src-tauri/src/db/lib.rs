use dotenv::dotenv;
use std::env;
use diesel::sqlite::SqliteConnection;
use diesel::prelude::*;
use crate::network::structs::{ResponseQuestion, QuestionList};
use crate::db::models::{NewQuestion, NewQuestionTopic};
use crate::db::schema::{question, question_topic};


pub fn db_connect() -> SqliteConnection {
    dotenv().ok();

    let db_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL not set in .env");
    SqliteConnection::establish(&db_url)
        .expect(&format!("Error connecting to {}", db_url))
}


pub async fn db_insert_all_lc_q_base_info(
    conn: &SqliteConnection, 
    response_questions: QuestionList
) -> Result<(), Box<dyn std::error::Error>> {
    // fetch all base leetcode question info over network and put it in db

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
        .execute(conn)?;
    diesel::insert_into(question_topic::table)
        .values(&db_new_question_topics)
        .execute(conn)?;

    Ok(())
}


fn db_format_question(
    response_question: &ResponseQuestion
) -> Result<NewQuestion, Box<dyn std::error::Error>> {
    // reshape struct for parsing response into struct for db insert
    // note that prompt is init to empty string. LC graphql api
    // only allows querying for prompt on individual questions

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


fn db_format_question_topics(
    response_question: ResponseQuestion
) -> Result<Vec<NewQuestionTopic>, Box<dyn std::error::Error>> {
    // reshape struct for parsing response into struct for db insert

    let mut question_topics: Vec<NewQuestionTopic> = vec![];
    for topic in response_question.topicTags {
        question_topics.push(NewQuestionTopic { 
            question_number: response_question.questionId.parse::<i32>()?, 
            topic: topic.name.clone(),
        })
    } 

    Ok(question_topics)
}