use diesel::sqlite::SqliteConnection;
use diesel::prelude::*;
use crate::network::fetchers::{fetch_lc_question_prompt, fetch_all_lc_questions};
use crate::db::models::Question;
use crate::db::lib::{
    insert_all_lc_q_base_info, 
    count_lc_questions_in_db,
    LEETCODE_SOURCE_ID
};

const INIT_FETCHED_PROMPTS: i32 = 20;

pub async fn get_lc_questions_on_init(
    conn: &SqliteConnection
) -> Result<(), Box<dyn std::error::Error>> {
    // check if db has LC questions inserted already, does fetch + insert if not

    if count_lc_questions_in_db(conn).await? > 0 { // questions previously inserted
        return Ok(());
    }

    let fetched_questions = fetch_all_lc_questions().await?;
    insert_all_lc_q_base_info(&conn, fetched_questions).await?;
    get_lc_question_prompts_on_init(&conn, INIT_FETCHED_PROMPTS).await?;

    Ok(())
}


async fn get_lc_question_prompts_on_init(
    conn: &SqliteConnection, 
    num_prompts: i32
) -> Result<(), Box<dyn std::error::Error>> {
    // get some question prompts for cards on top of deck when app starts up

    use crate::db::schema::question::dsl::*;
    let need_prompt_rows: Vec<Question> = question
        .filter(source.is_not_null())
        .filter(source.eq(LEETCODE_SOURCE_ID))
        .filter(source_qid.is_not_null())
        .filter(source_qid.lt(num_prompts + 1))
        .load(conn)?;

    // LC graphql api requires prompt fetching using title slug as graphql variable
    for q in need_prompt_rows {
        let prompt_html = fetch_lc_question_prompt(&q.title_slug.unwrap()).await?;
        diesel::update(question)
            .filter(qid.eq(q.qid))
            .set(prompt.eq(prompt_html))
            .execute(conn)?;
    }

    Ok(())
}


///////////////////////////////////////
////// ----- UNIT TESTS --------- /////
///////////////////////////////////////
#[cfg(test)]
mod test {
    use crate::db::lib::db_connect;
    use super::*;

    #[tokio::test]
    async fn get_lc_questions_on_init_test() {
        let conn = db_connect();

        let pre_call_qs = count_lc_questions_in_db(&conn).await.unwrap();
        let pre_call_prompts = count_lc_questions_with_prompts_in_db(&conn).await.unwrap();
        let call_success = get_lc_questions_on_init(&conn).await;
        assert!(call_success.is_ok());
        let post_call_qs = count_lc_questions_in_db(&conn).await.unwrap();
        let post_call_prompts = count_lc_questions_with_prompts_in_db(&conn).await.unwrap();

        if pre_call_qs > 0 {
            assert!(pre_call_qs > 0 && pre_call_qs == post_call_qs);
            assert!(pre_call_prompts > 0 && pre_call_prompts == post_call_prompts);
        }
        else {
            assert!(post_call_qs > 0);
            assert!(pre_call_prompts + (INIT_FETCHED_PROMPTS as i64) == post_call_prompts);
        }
    }


    async fn count_lc_questions_with_prompts_in_db(conn: &SqliteConnection) -> 
        Result<i64, Box<dyn std::error::Error>>  {
            use crate::db::schema::question::dsl::*;
            use diesel::dsl::not;

            let count = question
                .filter(source.eq(LEETCODE_SOURCE_ID))
                .filter(not(prompt.eq("")))
                .count()
                .first::<i64>(conn)?;
            
            Ok(count)
        }


}