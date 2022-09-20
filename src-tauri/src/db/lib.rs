use dotenv::dotenv;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use std::env;
use diesel::sqlite::SqliteConnection;
use diesel::prelude::*;
use crate::network::structs::{ResponseQuestion, QuestionList};
use crate::db::models::*;


pub const LEETCODE_SOURCE_ID: i32 = 1;
pub const SOURCELESS_QUESTION_SOURCE_ID: i32 = 0;
pub const TOPICLESS_QUESTION_TOPIC_ID: i32 = 0;

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

#[derive(Deserialize)]
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

#[derive(Serialize, Debug)]
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

pub struct FilterSets {
    diff: Option<HashSet<String>>,
    topics: Option<HashSet<i32>>,
    solved: Option<HashSet<bool>>,
    sources: Option<HashSet<i32>>,
    starred: Option<HashSet<bool>>,
    ranges: Option<Vec<(i32, i32)>>
}

pub async fn query_questions(options: QuestionOptions) -> Result<HashMap<i32, QuestionQueryResult>, Box<dyn std::error::Error>> { //Result<Vec<Question>, Box<dyn std::error::Error>> {
    if empty_query_option(&options) {
        return Ok(HashMap::new());
    }
    if options.range.is_some() && invalid_range(options.range.clone().unwrap()) {
        return Err("invalid range field".into());
    }

    let join_rows = join_question_soln_topic_star(options.user.clone())?;
    let filtered_questions = filter_question_soln_topic_join(options, join_rows);

    Ok(filtered_questions)
}

fn filter_question_soln_topic_join(options: QuestionOptions, join_rows: Vec<QuestionStarQTopicSolutionJoin>) -> HashMap<i32, QuestionQueryResult> {
    let QuestionOptions { 
        user: _, diff, topics, solved, source_ids, starred, range 
    } = options;

    let mut filter_sets = FilterSets {
        diff: None, topics: None, solved: None, sources: None, starred: None, ranges: None
    };
    if diff.is_some() { filter_sets.diff = Some(HashSet::from_iter(diff.unwrap())); }
    if topics.is_some() { filter_sets.topics = Some(HashSet::from_iter(topics.unwrap())); }
    if solved.is_some() { filter_sets.solved = Some(HashSet::from_iter(solved.unwrap())); }
    if source_ids.is_some() { filter_sets.sources = Some(HashSet::from_iter(source_ids.unwrap())); }
    if starred.is_some() { filter_sets.starred = Some(HashSet::from_iter(starred.unwrap())); }
    
    if range.is_some() {
        let mut sorted_ranges = range.unwrap();
        sorted_ranges.sort_by(|a, b| {
            let a_first = a.0 < b.0;
            if a_first { return std::cmp::Ordering::Less; }
            return std::cmp::Ordering::Greater;
        });
        filter_sets.ranges = Some(sorted_ranges);
    }
    let mut range_idx: usize = 0;
    let mut filtered_map: HashMap<i32, QuestionQueryResult> = HashMap::new();
    for row in join_rows.iter() {
        let (question_, star_, topic_, solution_) = row;
        
        // filter out questions not meeting criteria specified in options arg
        if filter_sets.ranges.is_some() {
            let num_ranges = filter_sets.ranges.as_ref().unwrap().len();
            while range_idx < num_ranges && question_.qid > filter_sets.ranges.as_ref().unwrap()[range_idx].1 {
                range_idx += 1;
                if range_idx == num_ranges {
                    break;
                }
            }
        }
        let out_of_range = filter_sets.ranges.is_some() &&  (
            range_idx == filter_sets.ranges.as_ref().unwrap().len() ||
            question_.qid < filter_sets.ranges.as_ref().unwrap()[range_idx].0);
        let bad_difficulty = filter_sets.diff.is_some() && 
            !filter_sets.diff.as_ref().unwrap().contains(
                &question_.difficulty.as_ref().unwrap().to_uppercase());
        let bad_topic = filter_sets.topics.is_some() && ((
            (topic_.is_none() && !filter_sets.topics.as_ref().unwrap().contains(&TOPICLESS_QUESTION_TOPIC_ID))) || 
            (topic_.is_some() && !filter_sets.topics.as_ref().unwrap().contains(&topic_.as_ref().unwrap().tid)));
        let bad_solve_status = filter_sets.solved.is_some() && (
            solution_.is_none() && !filter_sets.solved.as_ref().unwrap().contains(&false)) || 
            (solution_.is_some() && !filter_sets.solved.as_ref().unwrap().contains(&true));
        let bad_source = filter_sets.sources.is_some() && ((
            question_.source.is_some() && !filter_sets.sources.as_ref().unwrap().contains(&question_.source.unwrap())) ||
            (question_.source.is_none() && !filter_sets.sources.as_ref().unwrap().contains(&SOURCELESS_QUESTION_SOURCE_ID)));
        let bad_starred_status = filter_sets.starred.is_some() && (
            star_.is_none() && !filter_sets.starred.as_ref().unwrap().contains(&false)) || 
            (star_.is_some() && !filter_sets.starred.as_ref().unwrap().contains(&true));

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
                prompt: question_.prompt.clone(),
                difficulty: question_.difficulty.clone(),
                source: question_.source,
                source_qid: question_.source_qid
            };
            filtered_map.insert(question_.qid, new_q);
        }
        if topic_.is_some() {
            filtered_map.get_mut(&question_.qid).unwrap().topics.push(topic_.clone().unwrap().tid);
        }
        else {
            filtered_map.get_mut(&question_.qid).unwrap().topics.push(TOPICLESS_QUESTION_TOPIC_ID);
        }
    }

    filtered_map
}

fn empty_query_option(options: &QuestionOptions) -> bool {
    (options.diff.is_some() && options.diff.as_ref().unwrap().len() == 0) ||
    (options.topics.is_some() && options.topics.as_ref().unwrap().len() == 0) ||
    (options.solved.is_some() && options.solved.as_ref().unwrap().len() == 0) ||
    (options.source_ids.is_some() && options.source_ids.as_ref().unwrap().len() == 0) ||
    (options.starred.is_some() && options.starred.as_ref().unwrap().len() == 0) ||
    (options.range.is_some() && options.range.as_ref().unwrap().len() == 0)
}

fn join_question_soln_topic_star(uid: i32) -> Result<Vec<QuestionStarQTopicSolutionJoin>, Box<dyn std::error::Error>> {
    use crate::db::schema;
    use schema::question::dsl::*;
    let conn = db_connect();
    // get all potentially relevant rows containing quesiton info for a user.
    // simple and not too expensive. there will very rarely be more than
    // ~50000 rows here, and we are reading straight from disk with sqlite.
    // only care about one user at a time here, but need
    // all questions and their topics. there are 70 LC topics and 2300 questions, 
    // resulting in ~7000 rows on first init after install (if preload network 
    // call to LC hasn't failed). seems like num rows could get bad 
    // with max growth of O(questions * topics) by definition of left join
    // conditions. but realistically only ~3 topics per question, so avg rows 
    // is θ(questions * ~3). questions will rarely be over 5000 and topics 
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

fn invalid_range(mut ranges: Vec<(i32, i32)>) -> bool {
    ranges.sort_by(|a, b| {
        let a_first = a.0 < b.0 || (a.0 == b.0 && a.1 <= b.1);
        if a_first { return std::cmp::Ordering::Less; }
        return std::cmp::Ordering::Greater;
    });

    for i in 0..ranges.len() {
        let (start, stop) = ranges[i];
        if start > stop || (i > 0 && ranges[i - 1].1 >= start) { 
            return true; 
        }
    }

    false
}

///////////////////////////////////////
////// ----- UNIT TESTS --------- /////
///////////////////////////////////////

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn test_empty_query_option_none_fields() {
        let mut test_options = QuestionOptions {
            user: 1,
            diff: None,
            topics: Some(vec![1]),
            solved: Some(vec![true]),
            source_ids: Some(vec![1]),
            starred: Some(vec![true]),
            range: Some(vec![(1, 2)]),
        };
        assert_eq!(empty_query_option(&test_options), false);
        test_options.diff = Some(vec!["EASY".to_string()]);
        test_options.topics = None;
        assert_eq!(empty_query_option(&test_options), false);
        test_options.topics = Some(vec![1]);
        test_options.solved = None;
        assert_eq!(empty_query_option(&test_options), false);
        test_options.solved = Some(vec![true]);
        test_options.source_ids = None;
        assert_eq!(empty_query_option(&test_options), false);
        test_options.source_ids = Some(vec![1]);
        test_options.starred = None;
        assert_eq!(empty_query_option(&test_options), false);
        test_options.starred = Some(vec![true]);
        test_options.range = None;
        assert_eq!(empty_query_option(&test_options), false);
    }

    #[test]
    fn test_empty_query_option_zero_len_fields() {
        let mut test_options = QuestionOptions {
            user: 1,
            diff: Some(vec![]),
            topics: None,
            solved: None,
            source_ids: None,
            starred: None,
            range: None,
        };
        assert_eq!(empty_query_option(&test_options), true);
        test_options.diff = None;
        test_options.topics = Some(vec![]);
        assert_eq!(empty_query_option(&test_options), true);
        test_options.topics = None;
        test_options.solved = Some(vec![]);
        assert_eq!(empty_query_option(&test_options), true);
        test_options.solved = None;
        test_options.source_ids = Some(vec![]);
        assert_eq!(empty_query_option(&test_options), true);
        test_options.source_ids = None;
        test_options.starred = Some(vec![]);
        assert_eq!(empty_query_option(&test_options), true);
        test_options.starred = None;
        test_options.range = Some(vec![]);
        assert_eq!(empty_query_option(&test_options), true);
    }


    const TEST_QUESTIONS: i32 = 20;
    const EASY: &str = "EASY";
    const MEDIUM: &str = "MEDIUM";
    const HARD: &str = "HARD";
    #[test]
    fn test_filter_question_soln_topic_join_no_range_spec() {
        let test_options = QuestionOptions {
            user: 1,
            diff: None, topics: None, solved: None, 
            source_ids: None, starred: None, range: None,
        };

        let mut join_rows: Vec<QuestionStarQTopicSolutionJoin> = vec![];
        let mut expected_result_map: HashMap<i32, QuestionQueryResult> = HashMap::new();
        for test_qid in 1..TEST_QUESTIONS + 1 {
            let test_q = Question {
                qid: test_qid, 
                title: "test_question".to_string(), 

                title_slug: None, prompt: None, difficulty: Some(EASY.to_string()), 
                source: None, source_qid: None 
            };
            join_rows.push((test_q, None, None, None));

            let test_query_result = QuestionQueryResult {
                qid: test_qid,
                title: "test_question".to_string(),

                topics: vec![], starred: false, solved: false,
                title_slug: None, prompt: None, difficulty: Some(EASY.to_string()), 
                source: None, source_qid: None
            };
            expected_result_map.insert(test_qid, test_query_result);
        }

        let filter_result = filter_question_soln_topic_join(test_options, join_rows);
        assert!(filter_result.len() == TEST_QUESTIONS as usize);
        assert!(filter_result_hashmaps_match(filter_result, expected_result_map));
        
    }

    #[test]
    fn test_invalid_range_1() {
        let valid_range = vec![(2, 3), (1, 1), (100, 10000)];
        assert!(invalid_range(valid_range) == false);
    }
    #[test]
    fn test_invalid_range_2() {
        let invalid = vec![(1, 2), (4, 3)];
        assert!(invalid_range(invalid));
    }
    #[test]
    fn test_invalid_range_3() {
        let invalid = vec![(1, 4), (1, 6)];
        assert!(invalid_range(invalid));
    }

    #[test]
    fn test_filter_question_soln_topic_join_range_spec_1() {
        // NOTE: invalid ranges detected before the filterer gets called
        let test_range = vec![(1, 1), (4, 5), (9, 17)]; 
        let num_filtered = 1 + 2 + 9;
        let test_options = QuestionOptions {
            user: 1,
            diff: None, topics: None, solved: None, 
            source_ids: None, starred: None, 
            range: Some(test_range.clone()),
        };

        let mut join_rows: Vec<QuestionStarQTopicSolutionJoin> = vec![];
        let mut expected_result_map: HashMap<i32, QuestionQueryResult> = HashMap::new();
        for test_qid in 1..TEST_QUESTIONS + 1 {
            let test_q = Question {
                qid: test_qid, 
                title: "test_question".to_string(), 

                title_slug: None, prompt: None, difficulty: Some(EASY.to_string()), 
                source: None, source_qid: None 
            };
            join_rows.push((test_q, None, None, None));

            for (start, stop) in &test_range {
                if test_qid >= *start && test_qid <= *stop {
                    let test_query_result = QuestionQueryResult {
                        qid: test_qid,
                        title: "test_question".to_string(),

                        topics: vec![], starred: false, solved: false,
                        title_slug: None, prompt: None, difficulty: Some(EASY.to_string()), 
                        source: None, source_qid: None
                    };
                    expected_result_map.insert(test_qid, test_query_result);
                    break;
                }
            }
        }

        let filter_result = filter_question_soln_topic_join(test_options, join_rows);
        assert!(filter_result.len() == num_filtered);
        assert!(filter_result_hashmaps_match(filter_result, expected_result_map));
    }

    #[test]
    fn test_filter_question_soln_topic_join_include_sourceless() {
        let test_options = QuestionOptions {
            user: 1,
            source_ids: Some(vec![SOURCELESS_QUESTION_SOURCE_ID]),

            diff: None, topics: None, solved: None, starred: None, range: None,
        };

        let mut join_rows: Vec<QuestionStarQTopicSolutionJoin> = vec![];
        let mut expected_result_map: HashMap<i32, QuestionQueryResult> = HashMap::new();
        for test_qid in 1..TEST_QUESTIONS + 1 {
            let test_source_id = test_qid % 4;
            let test_q = Question {
                qid: test_qid, 
                title: "test_question".to_string(), 
                source: Some(test_source_id), 

                title_slug: None, prompt: None, difficulty: Some(EASY.to_string()), 
                source_qid: None,
            };
            join_rows.push((test_q, None, None, None));

            if test_source_id == SOURCELESS_QUESTION_SOURCE_ID {
                let test_query_result = QuestionQueryResult {
                    qid: test_qid,
                    title: "test_question".to_string(),
                    source: Some(test_source_id),
    
                    topics: vec![], starred: false, solved: false,
                    title_slug: None, prompt: None, difficulty: Some(EASY.to_string()), 
                    source_qid: None, 
                };
                expected_result_map.insert(test_qid, test_query_result);
            }
        }

        let filter_result = filter_question_soln_topic_join(test_options, join_rows);
        assert!(filter_result.len() as i32 == TEST_QUESTIONS / 4);
        assert!(filter_result_hashmaps_match(filter_result, expected_result_map));
    }

    #[test]
    fn test_filter_question_soln_topic_join_include_topicless() {
        let test_options = QuestionOptions {
            user: 1,
            topics: Some(vec![TOPICLESS_QUESTION_TOPIC_ID]),

            diff: None, source_ids: None, solved: None, starred: None, range: None,
        };

        let mut join_rows: Vec<QuestionStarQTopicSolutionJoin> = vec![];
        let mut expected_result_map: HashMap<i32, QuestionQueryResult> = HashMap::new();
        for test_qid in 1..TEST_QUESTIONS + 1 {
            let test_topic_id = test_qid % 4;
            let test_q = Question {
                qid: test_qid, 
                title: "test_question".to_string(), 

                title_slug: None, prompt: None, difficulty: Some(EASY.to_string()), 
                source_qid: None, source: None, 
            };
            let test_qt = QuestionTopic {
                relid: test_qid,
                qid: test_qid,
                tid: test_topic_id
            };
            
            if test_topic_id == TOPICLESS_QUESTION_TOPIC_ID {
                let test_query_result = QuestionQueryResult {
                    qid: test_qid,
                    title: "test_question".to_string(),
                    source: Some(test_topic_id),
    
                    topics: vec![], starred: false, solved: false,
                    title_slug: None, prompt: None, difficulty: Some(EASY.to_string()), 
                    source_qid: None, 
                };

                expected_result_map.insert(test_qid, test_query_result);
                join_rows.push((test_q, None, None, None));
            }
            else {
                join_rows.push((test_q.clone(), None, Some(test_qt), None));
            }
        }

        let filter_result = filter_question_soln_topic_join(test_options, join_rows);
        assert!(filter_result.len() as i32 == TEST_QUESTIONS / 4);
        assert!(filter_result_hashmaps_match(filter_result, expected_result_map));
    }

    #[test]
    fn test_filter_question_soln_topic_join_filter_difficulty() {
        const FILTERED_DIFFS: [&str; 2] = [EASY, HARD];
        const DIFFS: [&str; 3] = [EASY, MEDIUM, HARD];
        let test_options = QuestionOptions {
            user: 1,
            diff: Some(FILTERED_DIFFS.map(|s| s.to_string()).to_vec()), 

            source_ids: None, solved: None, starred: None, range: None, topics: None,
        };

        let mut join_rows: Vec<QuestionStarQTopicSolutionJoin> = vec![];
        let mut expected_result_map: HashMap<i32, QuestionQueryResult> = HashMap::new();
        let mut diff_idx = 0;
        let filtered_diff_set = HashSet::from(FILTERED_DIFFS);
        let mut num_filtered = 0;
        for test_qid in 1..TEST_QUESTIONS + 1 {
            let test_q = Question {
                qid: test_qid, 
                title: "test_question".to_string(), 
                difficulty: Some(DIFFS[diff_idx].to_string()),

                title_slug: None, prompt: None,  
                source_qid: None, source: None, 
            };

            join_rows.push((test_q, None, None, None));
            if filtered_diff_set.contains(DIFFS[diff_idx]) {
                num_filtered += 1;
                let test_query_result = QuestionQueryResult {
                    qid: test_qid,
                    title: "test_question".to_string(),
                    difficulty: Some(DIFFS[diff_idx].to_string()), 

                    topics: vec![], starred: false, solved: false,
                    title_slug: None, prompt: None, 
                    source_qid: None, source: None,
                };
                expected_result_map.insert(test_qid, test_query_result);
            }

            diff_idx += 1;
            if diff_idx == DIFFS.len() {
                diff_idx = 0;
            }
        }

        let filter_result = filter_question_soln_topic_join(test_options, join_rows);
        assert!(filter_result.len() == num_filtered);
        assert!(filter_result_hashmaps_match(filter_result, expected_result_map));
    }

    #[test]
    fn test_filter_question_soln_topic_join_filter_topic() {
        const FILTERED_TOPICS: [i32; 3]= [TOPICLESS_QUESTION_TOPIC_ID, 1, 2];
        let test_options = QuestionOptions {
            user: 1,
            topics: Some(Vec::from(FILTERED_TOPICS)),

            diff: None, source_ids: None, solved: None, starred: None, range: None,
        };

        let mut join_rows: Vec<QuestionStarQTopicSolutionJoin> = vec![];
        let mut expected_result_map: HashMap<i32, QuestionQueryResult> = HashMap::new();
        let mut num_filtered = 0;
        for test_qid in 1..TEST_QUESTIONS + 1 {
            let test_topic_id = test_qid % 5;
            let test_q = Question {
                qid: test_qid, 
                title: "test_question".to_string(), 

                title_slug: None, prompt: None, difficulty: Some(EASY.to_string()), 
                source_qid: None, source: None, 
            };
            let test_qt = QuestionTopic {
                relid: test_qid,
                qid: test_qid,
                tid: test_topic_id
            };
            
            if FILTERED_TOPICS.contains(&test_topic_id) {
                num_filtered += 1;
                let test_query_result = QuestionQueryResult {
                    qid: test_qid,
                    title: "test_question".to_string(),
                    source: Some(test_topic_id),
    
                    topics: vec![], starred: false, solved: false,
                    title_slug: None, prompt: None, difficulty: Some(EASY.to_string()), 
                    source_qid: None, 
                };

                expected_result_map.insert(test_qid, test_query_result);
                join_rows.push((test_q, None, None, None));
            }
            else {
                join_rows.push((test_q.clone(), None, Some(test_qt), None));
            }
        }

        let filter_result = filter_question_soln_topic_join(test_options, join_rows);
        assert!(num_filtered == expected_result_map.len() as i32);
        assert!(filter_result_hashmaps_match(filter_result, expected_result_map));
    }

    #[test]
    fn test_filter_question_soln_topic_join_filter_solved() {
        let test_options = QuestionOptions {
            user: 1,
            solved: Some(vec![true]),

            diff: None, topics: None,  
            source_ids: None, starred: None, range: None,
        };

        let mut join_rows: Vec<QuestionStarQTopicSolutionJoin> = vec![];
        let mut expected_result_map: HashMap<i32, QuestionQueryResult> = HashMap::new();
        for test_qid in 1..TEST_QUESTIONS + 1 {
            let test_q = Question {
                qid: test_qid, 
                title: "test_question".to_string(), 

                title_slug: None, prompt: None, difficulty: Some(EASY.to_string()), 
                source: None, source_qid: None 
            };
            
            if test_qid % 2 == 1 {
                let test_solution = Solution {
                    sid: test_qid,
                    uid: 1,
                    qid: test_qid,
                    notes: "test notes".to_string()
                };
                let test_query_result = QuestionQueryResult {
                    qid: test_qid,
                    title: "test_question".to_string(),
                    solved: true,

                    topics: vec![], starred: false,
                    title_slug: None, prompt: None, difficulty: Some(EASY.to_string()), 
                    source: None, source_qid: None
                };
                expected_result_map.insert(test_qid, test_query_result);
                join_rows.push((test_q, None, None, Some(test_solution)));
            }
            else {
                join_rows.push((test_q, None, None, None));
            }
        }

        let filter_result = filter_question_soln_topic_join(test_options, join_rows);
        assert!(filter_result.len() as i32 == TEST_QUESTIONS / 2);
        assert!(filter_result_hashmaps_match(filter_result, expected_result_map));
    }

    #[test]
    fn test_filter_question_soln_topic_join_filter_source() {
        const FILTERED_SOURCES: [i32; 3] = [SOURCELESS_QUESTION_SOURCE_ID, 1, 2];
        let test_options = QuestionOptions {
            user: 1,
            source_ids: Some(Vec::from(FILTERED_SOURCES)),

            diff: None, topics: None, solved: None, starred: None, range: None,
        };

        let mut join_rows: Vec<QuestionStarQTopicSolutionJoin> = vec![];
        let mut expected_result_map: HashMap<i32, QuestionQueryResult> = HashMap::new();
        let mut num_filtered = 0;
        for test_qid in 1..TEST_QUESTIONS + 1 {
            let test_source_id = test_qid % 5;
            let test_q = Question {
                qid: test_qid, 
                title: "test_question".to_string(), 
                source: Some(test_source_id), 

                title_slug: None, prompt: None, difficulty: Some(EASY.to_string()), 
                source_qid: None,
            };
            join_rows.push((test_q, None, None, None));

            if FILTERED_SOURCES.contains(&test_source_id) {
                num_filtered += 1;
                let test_query_result = QuestionQueryResult {
                    qid: test_qid,
                    title: "test_question".to_string(),
                    source: Some(test_source_id),
    
                    topics: vec![], starred: false, solved: false,
                    title_slug: None, prompt: None, difficulty: Some(EASY.to_string()), 
                    source_qid: None, 
                };
                expected_result_map.insert(test_qid, test_query_result);
            }
        }

        let filter_result = filter_question_soln_topic_join(test_options, join_rows);
        assert!(filter_result.len() as i32 == num_filtered);
        assert!(filter_result_hashmaps_match(filter_result, expected_result_map));
    }

    #[test]
    fn test_filter_question_soln_topic_join_filter_starred() {
        let test_options = QuestionOptions {
            user: 1,
            starred: Some(vec![true]),

            diff: None, topics: None,  
            source_ids: None, solved: None, range: None,
        };

        let mut join_rows: Vec<QuestionStarQTopicSolutionJoin> = vec![];
        let mut expected_result_map: HashMap<i32, QuestionQueryResult> = HashMap::new();
        for test_qid in 1..TEST_QUESTIONS + 1 {
            let test_q = Question {
                qid: test_qid, 
                title: "test_question".to_string(), 

                title_slug: None, prompt: None, difficulty: Some(EASY.to_string()), 
                source: None, source_qid: None 
            };
            
            if test_qid % 2 == 0 {
                let test_star = Star {
                    relid: test_qid,
                    uid: 1,
                    qid: test_qid,
                };
                let test_query_result = QuestionQueryResult {
                    qid: test_qid,
                    title: "test_question".to_string(),
                    starred: true,

                    topics: vec![], solved: false,
                    title_slug: None, prompt: None, difficulty: Some(EASY.to_string()), 
                    source: None, source_qid: None
                };
                expected_result_map.insert(test_qid, test_query_result);
                join_rows.push((test_q, Some(test_star), None, None));
            }
            else {
                join_rows.push((test_q, None, None, None));
            }
        }

        let filter_result = filter_question_soln_topic_join(test_options, join_rows);
        assert!(filter_result.len() as i32 == TEST_QUESTIONS / 2);
        assert!(filter_result_hashmaps_match(filter_result, expected_result_map));
    }

    fn filter_result_hashmaps_match(map1: HashMap<i32, QuestionQueryResult>, map2: HashMap<i32, QuestionQueryResult>) -> bool {
        map1.len() == map2.len() && map1.keys().all(|k| map2.contains_key(k))
    }

}