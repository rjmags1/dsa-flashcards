use diesel::sqlite::SqliteConnection;
use diesel::prelude::*;
use crate::network::fetchers::{fetch_prompt, fetch_all_questions};
use crate::db::models::Question;
use crate::db::lib::{db_insert_all_lc_q_base_info, LEETCODE_SOURCE_ID};


async fn get_init_prompts(
    conn: &SqliteConnection, 
    num_prompts: i32
) -> Result<(), Box<dyn std::error::Error>> {
    // get some question prompts for cards on top of deck

    use crate::db::schema::question::dsl::*;
    let need_prompt_rows: Vec<Question> = question
        .filter(source.is_not_null())
        .filter(source.eq(LEETCODE_SOURCE_ID))
        .filter(source_qid.is_not_null())
        .filter(source_qid.lt(num_prompts + 1))
        .load(conn)?;

    // LC graphql api requires prompt fetching using title slug as graphql variable
    for q in need_prompt_rows {
        let prompt_html = fetch_prompt(&q.title_slug.unwrap()).await?;
        diesel::update(question)
            .filter(qid.eq(q.qid))
            .set(prompt.eq(prompt_html))
            .execute(conn)?;
    }

    Ok(())
}


pub async fn get_lc_questions_on_init(
    conn: SqliteConnection
) -> Result<(), Box<dyn std::error::Error>> {
    // check if db has LC questions inserted already, does fetch + insert if not
    use crate::db::schema::question::dsl::*;

    let an_lc_q = question
        .filter(source.eq(LEETCODE_SOURCE_ID))
        .first::<Question>(&conn);
    if an_lc_q.is_ok() { // questions previously inserted
        return Ok(());
    }

    let fetched_questions = fetch_all_questions().await?;
    db_insert_all_lc_q_base_info(&conn, fetched_questions).await?;
    get_init_prompts(&conn, 20).await?;

    Ok(())
}