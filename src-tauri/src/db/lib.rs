use dotenv::dotenv;
use std::collections::HashMap;
use std::env;
use diesel::sqlite::SqliteConnection;
use diesel::prelude::*;
use crate::network::structs::{ResponseQuestion, QuestionList};
use crate::db::models::*;


pub const LEETCODE_SOURCE_ID: i32 = 1;

pub fn db_connect() -> SqliteConnection {
    dotenv().ok();

    let db_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL not set in .env");
    SqliteConnection::establish(&db_url)
        .expect(&format!("Error connecting to {}", db_url))
}


pub async fn insert_all_lc_q_base_info(
    conn: &SqliteConnection, 
    response_questions: QuestionList
) -> Result<(), Box<dyn std::error::Error>> {
    // fetch all base leetcode question info over network and put it in db

    let mut db_new_questions: Vec<NewQuestion> = vec![];
    for response_question in &response_questions.questions {
        db_new_questions.push(
            db_format_question(response_question, LEETCODE_SOURCE_ID)?);
    }
    insert_new_questions(db_new_questions, conn).await?;

    let mut db_new_question_topics: Vec<NewQuestionTopic> = vec![];
    let mut all_topics = select_all_topics_into_map(conn).await?;
    for rq in response_questions.questions {
        let new_topics: Vec<NewQuestionTopic> = db_format_question_topics(
            rq, Some(LEETCODE_SOURCE_ID), None, conn, &mut all_topics).await?;
        for topic in new_topics {
            db_new_question_topics.push(topic);
        }
    }
    insert_new_question_topics(db_new_question_topics, conn).await?;

    Ok(())
}


fn db_format_question(
    response_question: &ResponseQuestion,
    src_id: i32
) -> Result<NewQuestion, Box<dyn std::error::Error>> {
    // reshape struct for parsing response into struct for db insert
    // note that prompt is init to empty string. LC graphql api
    // only allows querying for prompt on individual questions

    let new_question = NewQuestion {
        title: response_question.title.clone(),
        title_slug: response_question.titleSlug.clone(),
        difficulty: response_question.difficulty.to_uppercase(),
        prompt: "".to_string(),
        source: src_id,
        source_qid: response_question.questionId.parse::<i32>()?,
    };

    Ok(new_question)
}


async fn db_format_question_topics(
    response_question: ResponseQuestion,
    src_id: Option<i32>,
    db_qid: Option<i32>,
    conn: &SqliteConnection,
    all_topics: &mut HashMap<String, i32>
) -> Result<Vec<NewQuestionTopic>, Box<dyn std::error::Error>> {
    // reshape struct for parsing response into struct for db insert
    if src_id.is_none() && db_qid.is_none() {
        return Err("must provide src_id or db_qid".into());
    }

    let mut question_topics: Vec<NewQuestionTopic> = vec![];
    let src_qid = response_question.questionId.parse::<i32>()?;
    let need_determine_db_qid = db_qid.is_none();
    for topic in response_question.topicTags {
        let qid: i32;
        if need_determine_db_qid {
            qid = select_qid_from_src_info(src_id.unwrap(), src_qid, conn).await?;
        }
        else { qid = db_qid.unwrap(); }

        let tid: i32;
        if all_topics.contains_key(&topic.name) {
            tid = *all_topics.get(&topic.name).unwrap();
        }
        else {
            tid = insert_new_topic(&topic.name, conn).await?.tid;
            all_topics.insert(topic.name, tid);
        }

        question_topics.push(NewQuestionTopic { qid, tid })
    } 

    Ok(question_topics)
}

async fn select_qid_from_src_info(
    src_id: i32, 
    src_qid: i32, 
    conn: &SqliteConnection
) -> Result<i32, Box<dyn std::error::Error>> {
    // queries db and determines qid (db primary key) based 
    // on src info using Question struct
    use crate::db::schema::question::dsl::*;

    let the_qid = question
        .filter(source_qid.eq(src_qid))
        .filter(source.eq(src_id))
        .select(qid)
        .first::<i32>(conn)?;

    Ok(the_qid)
}

async fn select_all_topics_into_map(conn: &SqliteConnection) -> 
Result<HashMap<String, i32>, Box<dyn std::error::Error>> {
    // gets all distinct topic strings from db, reading into Topic struct
    use crate::db::schema::topic::dsl::*; 

    let topic_rows: Vec<Topic> = topic.load(conn)?;
    let mut topic_to_id: HashMap<String, i32> = HashMap::new();
    for t in topic_rows {
        topic_to_id.insert(t.name, t.tid);
    }

    Ok(topic_to_id)
}

async fn insert_new_topic(
    new_name: &String, 
    conn: &SqliteConnection
) -> Result<Topic, Box<dyn std::error::Error>> {
    // insert a new topic into the db using the NewTopic struct
    use crate::db::schema::topic::dsl::*; 

    let nt = NewTopic { name: new_name.clone() };
    diesel::insert_into(topic)
        .values(&nt)
        .execute(conn)?;

    let inserted: Topic = topic
        .order_by(tid.desc())
        .first(conn)?;
    
    Ok(inserted)
}

async fn insert_new_question_topics(
    ts: Vec<NewQuestionTopic>, 
    conn: &SqliteConnection
) -> Result<Vec<QuestionTopic>, Box<dyn std::error::Error>> {
    // inserts multiple topic-question M:M relationships into the 
    // db using NewQuestionTopic struct
    use crate::db::schema::question_topic::dsl::*;

    let num_new: i64 = ts.len().try_into().unwrap();
    diesel::insert_into(question_topic)
        .values(&ts)
        .execute(conn)?;
    
    let inserted: Vec<QuestionTopic> = question_topic
        .order_by(relid.desc())
        .limit(num_new)
        .load::<QuestionTopic>(conn)?;
    
    Ok(inserted)
}

async fn insert_new_questions(
    qs: Vec<NewQuestion>, 
    conn: &SqliteConnection
) -> Result<Vec<Question>, Box<dyn std::error::Error>> { 
    // inserts multiple questions into db using NewQuestion struct
    use crate::db::schema::question::dsl::*;
    
    let num_new: i64 = qs.len().try_into().unwrap();
    diesel::insert_into(question)
        .values(qs)
        .execute(conn)?;
    
    let inserted: Vec<Question> = question
        .order_by(qid.desc())
        .limit(num_new)
        .load::<Question>(conn)?;
    
    Ok(inserted)
}

pub async fn count_lc_questions_in_db(conn: &SqliteConnection) -> 
Result<i64, Box<dyn std::error::Error>>  {
    use crate::db::schema::question::dsl::*;
    let count = question
        .filter(source.eq(LEETCODE_SOURCE_ID))
        .count()
        .first::<i64>(conn)?;
    
    Ok(count)
}



///////////////////////////////////////
////// ----- UNIT TESTS --------- /////
///////////////////////////////////////

// note: at this point don't see a need to write unit tests here.
//       functions either map structs to other structs or wrap
//       diesel queries.