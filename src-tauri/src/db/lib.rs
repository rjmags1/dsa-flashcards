use dotenv::dotenv;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::env;
use diesel::sqlite::SqliteConnection;
use diesel::prelude::*;
use crate::network::structs::{ResponseQuestion, QuestionList};
use crate::db::models::*;


pub const LEETCODE_SOURCE_ID: i32 = 1;
pub const SOURCELESS_QUESTION: i32 = 0;
pub const TOPICLESS_QUESTION: i32 = 0;

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

pub struct QuestionOptions {
    user: i32,
    diff: Option<Vec<String>>,
    topics: Option<Vec<i32>>,//X
    solved: Option<Vec<bool>>,//X
    source_ids: Option<Vec<i32>>,
    starred: Option<Vec<bool>>,//X
    range: Option<Vec<(i32, i32)>>,
}

type QuestionStarQTopicSolutionJoin = (
    Question, 
    Option<Star>, 
    Option<QuestionTopic>, 
    Option<Solution>
);

#[derive(Serialize)]
pub struct QuestionQueryResult {
    qid: i32,
    starred: bool,
    solved: bool,
    topics: Vec<i32>,
    title: String,
    title_slug: Option<String>,
    prompt: Option<String>,
    difficulty: Option<String>,
    source: Option<i32>,
    source_qid: Option<i32>,
}

pub async fn query_questions(options: QuestionOptions) -> Result<HashMap<i32, QuestionQueryResult>, Box<dyn std::error::Error>> { //Result<Vec<Question>, Box<dyn std::error::Error>> {
    if empty_query_option(&options) {
        return Ok(HashMap::new());
    }

    let join_rows = join_question_soln_topic_star(options.user.clone())?;
    let filtered_questions = filter_question_soln_topic_join(options, join_rows);

    Ok(filtered_questions)
}

fn filter_question_soln_topic_join(options: QuestionOptions, join_rows: Vec<QuestionStarQTopicSolutionJoin>) -> HashMap<i32, QuestionQueryResult> {
    let QuestionOptions { 
        user: _, diff, topics, solved, source_ids, starred, range 
    } = options;

    let diff_set: HashSet<String> = HashSet::from_iter(diff.unwrap());
    let topic_set: HashSet<i32> = HashSet::from_iter(topics.unwrap());
    let solved_set: HashSet<bool> = HashSet::from_iter(solved.unwrap());
    let source_id_set: HashSet<i32> = HashSet::from_iter(source_ids.unwrap());
    let starred_set: HashSet<bool> = HashSet::from_iter(starred.unwrap());
    let mut sorted_ranges = range.unwrap();
    sorted_ranges.sort_by(|a, b| {
        let a_first = a.0 < b.0 || (a.0 == b.0 && a.1 <= b.1);
        if a_first { return std::cmp::Ordering::Less; }
        return std::cmp::Ordering::Greater;
    });
    let ranges_len = sorted_ranges.len();
    let mut range_idx: usize = 0;
    let mut filtered_map: HashMap<i32, QuestionQueryResult> = HashMap::new();
    for row in join_rows.iter() {
        let (question_, star_, topic_, solution_) = row;
        
        // filter out questions not meeting criteria specified in options arg
        while range_idx < ranges_len && question_.qid > sorted_ranges[range_idx].1 {
            range_idx += 1;
            if range_idx == ranges_len {
                break;
            }
        }
        let out_of_range = question_.qid < sorted_ranges[range_idx].0;
        let bad_difficulty = !diff_set.contains(
            &question_.difficulty.as_ref().unwrap().to_uppercase());
        let bad_topic = (
            topic_.is_none() && !topic_set.contains(&TOPICLESS_QUESTION)) || 
            !topic_set.contains(&topic_.as_ref().unwrap().tid);
        let bad_solve_status = (
            solution_.is_none() && !solved_set.contains(&false)) || 
            (solution_.is_some() && !solved_set.contains(&true));
        let bad_source = (
            question_.source.is_some() && !source_id_set.contains(&question_.source.unwrap())) ||
            (question_.source.is_none() && !source_id_set.contains(&SOURCELESS_QUESTION));
        let bad_starred_status = (
            star_.is_none() && !starred_set.contains(&false)) || 
            (star_.is_some() && !starred_set.contains(&true));

        if out_of_range || bad_difficulty || bad_topic || 
            bad_solve_status || bad_source || bad_starred_status {
            continue;
        }
        
        if !filtered_map.contains_key(&question_.qid) {
            let new_q = QuestionQueryResult {
                qid: question_.qid,
                starred: star_.is_some(),
                solved: solution_.is_some(),
                topics: vec![],
                title: question_.title.clone(),
                title_slug: question_.title_slug.clone(),
                prompt: question_.title_slug.clone(),
                difficulty: question_.difficulty.clone(),
                source: question_.source,
                source_qid: question_.source_qid
            };
            filtered_map.insert(question_.qid, new_q);
        }
        if topic_.is_some() {
            filtered_map.get_mut(&question_.qid).unwrap().topics.push(topic_.clone().unwrap().relid);
        }
    }

    filtered_map
}

fn empty_query_option(options: &QuestionOptions) -> bool {
    options.diff.is_none() || 
    options.topics.is_none() || 
    options.solved.is_none() || 
    options.source_ids.is_none() || 
    options.starred.is_none() || 
    options.diff.as_ref().unwrap().len() == 0 ||
    options.topics.as_ref().unwrap().len() == 0 ||
    options.solved.as_ref().unwrap().len() == 0 ||
    options.source_ids.as_ref().unwrap().len() == 0 ||
    options.starred.as_ref().unwrap().len() == 0 ||
    (options.range.is_some() && options.range.as_ref().unwrap().len() == 0)
}

fn join_question_soln_topic_star(uid: i32) -> Result<Vec<QuestionStarQTopicSolutionJoin>, Box<dyn std::error::Error>> {
    use crate::db::schema;
    use schema::question::dsl::*;
    let conn = db_connect();
    // get all potentially relevant rows containing quesiton info for a user.
    // simple and not too expensive. there will very rarely be more than
    // ~50000k rows here, and we are reading straight from disk with sqlite.
    // only care about one user at a time here, but need
    // all questions and their topics. there are 70 LC topics and 2300 questions, 
    // resulting in ~7000 rows on first init after install (if preload network 
    // call to LC hasn't failed). seems like num rows could get bad 
    // with max growth of O(questions * topics) by definition of left join
    // conditions. but realistically only ~3 topics per question, so avg row 
    // θ(questions * ~3). questions will rarely be over 5000 and topics 
    // may grow but the trend of 3 topics/questions should hold well.

    let join_rows: Vec<QuestionStarQTopicSolutionJoin> = question
        .left_outer_join(schema::star::table.on(
            schema::star::qid.eq(qid).and(
            // ignore star rows w/ uid != user during join
            schema::star::uid.eq(uid)) 
        ))
        .left_outer_join(schema::question_topic::table)
        .left_outer_join(schema::solution::table.on(
            schema::solution::qid.eq(qid).and(
            // ignore solution rows w/ uid != user during join
            schema::solution::uid.eq(uid)) 
        ))
        .load(&conn)?;

    Ok(join_rows)
}

///////////////////////////////////////
////// ----- UNIT TESTS --------- /////
///////////////////////////////////////

// note: at this point don't see a need to write unit tests here.
//       functions either map structs to other structs or wrap
//       diesel queries.

mod test {
    //use super::*;
    
//    #[test]
    //fn test_empty_query_option_all_none() {
    //}
    //#[test]
    //fn test_empty_query_option_all_zero_len() {
    //}
    //#[test]
    //fn test_empty_query_option_none_and_zero_len() {
    //}
    //#[test]
    //fn test_empty_query_option_non_empty() {
    //}


    //#[test]
    //fn test_filter_question_soln_topic_join_range_none() {
    //}
    //#[test]
    //fn test_filter_question_soln_topic_join_range_empty_vec() {
    //}
    //#[test]
    //fn test_filter_question_soln_topic_join_range_nonempty_vec_1() {
    //}
    //#[test]
    //fn test_filter_question_soln_topic_join_range_nonempty_vec_2() {
    //}
    //#[test]
    //fn test_filter_question_soln_topic_join_include_sourceless() {
    //}
    //#[test]
    //fn test_filter_question_soln_topic_join_include_topicless() {
    //}
    //#[test]
    //fn test_filter_question_soln_topic_join_filter_difficulty() {
    //}
    //#[test]
    //fn test_filter_question_soln_topic_join_filter_topic() {
    //}
    //#[test]
    //fn test_filter_question_soln_topic_join_filter_solved() {
    //}
    //#[test]
    //fn test_filter_question_soln_topic_join_filter_source() {
    //}
    //#[test]
    //fn test_filter_question_soln_topic_join_filter_starred() {
    //}

}