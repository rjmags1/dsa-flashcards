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
    solution (sid) {
        sid -> Integer,
        uid -> Integer,
        qid -> Integer,
        notes -> Text,
    }
}

table! {
    source (sid) {
        sid -> Integer,
        name -> Text,
    }
}

table! {
    star (relid) {
        relid -> Integer,
        qid -> Integer,
        uid -> Integer,
    }
}

table! {
    topic (tid) {
        tid -> Integer,
        name -> Text,
    }
}

table! {
    user (uid) {
        uid -> Integer,
        name -> Text,
        hide_diff -> Nullable<Bool>,
        hide_cat -> Nullable<Bool>,
        hide_solved -> Nullable<Bool>,
        dark_mode -> Nullable<Bool>,
    }
}

joinable!(question -> source (source));
joinable!(question_topic -> question (qid));
joinable!(question_topic -> topic (tid));
joinable!(solution -> question (qid));
joinable!(solution -> user (uid));
joinable!(star -> question (qid));
joinable!(star -> user (uid));

allow_tables_to_appear_in_same_query!(
    question,
    question_topic,
    solution,
    source,
    star,
    topic,
    user,
);
