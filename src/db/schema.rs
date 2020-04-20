table! {
    use diesel::sql_types::*;

    backs (id) {
        id -> Int4,
        text -> Text,
        language -> Int4,
        audio -> Nullable<Text>,
        image -> Nullable<Text>,
    }
}

table! {
    use diesel::sql_types::*;

    cards (id) {
        id -> Int4,
        created_at -> Int8,
        front -> Text,
        back -> Int4,
        deck -> Int4,
        link -> Nullable<Text>,
    }
}

table! {
    use diesel::sql_types::*;

    decks (id) {
        id -> Int4,
        name -> Varchar,
        owner -> Int4,
        language -> Int4,
    }
}

table! {
    use diesel::sql_types::*;

    languages (id) {
        id -> Int4,
        name -> Varchar,
        abbreviation -> Varchar,
    }
}

table! {
    use diesel::sql_types::*;

    scores (id) {
        id -> Int4,
        created_at -> Int8,
        card -> Int4,
        value -> Int2,
    }
}

table! {
    use diesel::sql_types::*;

    set_cards (id) {
        id -> Int4,
        card_id -> Int4,
        set_id -> Int4,
    }
}

table! {
    use diesel::sql_types::*;

    sets (id) {
        id -> Int4,
        created_at -> Int8,
        name -> Text,
        deck -> Int4,
        owner -> Int4,
    }
}

table! {
    use diesel::sql_types::*;

    users (id) {
        id -> Int4,
        username -> Varchar,
        password -> Varchar,
        created_at -> Int8,
        updated_at -> Int8,
    }
}

joinable!(backs -> languages (language));
joinable!(cards -> backs (back));
joinable!(cards -> decks (deck));
joinable!(decks -> languages (language));
joinable!(decks -> users (owner));
joinable!(scores -> cards (card));
joinable!(set_cards -> cards (card_id));
joinable!(set_cards -> sets (set_id));
joinable!(sets -> decks (deck));
joinable!(sets -> users (owner));

allow_tables_to_appear_in_same_query!(
    backs,
    cards,
    decks,
    languages,
    scores,
    set_cards,
    sets,
    users,
);
