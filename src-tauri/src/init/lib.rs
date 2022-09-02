use diesel::sqlite::SqliteConnection;
use diesel::prelude::*;
use crate::network::fetchers::{fetch_prompt, fetch_all_questions};
use crate::db::models::Question;
use crate::db::lib::db_insert_all_lc_q_base_info;


async fn get_init_prompts(
    conn: &SqliteConnection, 
    num_prompts: i32
) -> Result<(), Box<dyn std::error::Error>> {
    // get some question prompts for cards on top of deck

    use crate::db::schema::question::dsl::*;
    let title_slugs: Vec<(String, i32)> = question
        .filter(fetched)
        .filter(question_number.lt(num_prompts + 1))
        .select((title_slug, id))
        .load(conn)?;

    // LC graphql api requires prompt fetching using title slug as graphql variable
    for (slug, q_id) in title_slugs {
        let prompt_html = fetch_prompt(&slug).await?;
        diesel::update(question)
            .filter(id.eq(q_id))
            .set(prompt.eq(prompt_html))
            .execute(conn)?;
    }
    Ok(())
}


pub async fn get_lc_questions_on_init(
    conn: SqliteConnection
) -> Result<(), Box<dyn std::error::Error>> {
    // check if db has LC questions inserted already, does fetch + insert if not

    let query_string = "SELECT * FROM question WHERE fetched = TRUE LIMIT 1;";
    let q_rows: Vec<Question> =  diesel::sql_query(query_string).load(&conn)?;
    if q_rows.len() == 1 { // questions previously inserted
        return Ok(());
    }

    let fetched_questions = fetch_all_questions().await?;
    db_insert_all_lc_q_base_info(&conn, fetched_questions).await?;
    get_init_prompts(&conn, 20).await?;

    Ok(())
}