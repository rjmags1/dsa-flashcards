table! {
    question (qid) {
        qid -> Integer,
        title -> Text,
        title_slug -> Nullable<Text>,
        prompt -> Nullable<Text>,
        difficulty -> Nullable<Text>,
        source -> Nullable<Integer>,
        source_qid -> Nullable<Integer>,
    }
}

table! {
    question_topic (relid) {
        relid -> Integer,
        qid -> Integer,
        tid -> Integer,
    }
}

table! {
    source (sid) {
        sid -> Integer,
        name -> Text,
    }
}

table! {
    topic (tid) {
        tid -> Integer,
        name -> Text,
    }
}

joinable!(question -> source (source));

allow_tables_to_appear_in_same_query!(
    question,
    question_topic,
    source,
    topic,
);
