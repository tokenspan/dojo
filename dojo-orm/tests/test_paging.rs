use chrono::{NaiveDateTime, Utc};
use googletest::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use common::*;
use dojo_macros::Model;
use dojo_orm::Database;

mod common;

macro_rules! create_users {
    ($db: ident, names = $($name:literal),+) => {
        $($db.insert(&[&User {
            id: Uuid::new_v4(),
            name: $name.to_string(),
            email: concat!($name, "@gmail.com").to_string(),
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        }]).all().await?;)+
    };
}

macro_rules! create_paging_args {
    (first = $first: literal) => {
        (Some($first as i64), None)
    };
    (first = $first: literal, after = $after: ident) => {
        (Some($first as i64), Some($after))
    };
    (last = $last: literal) => {
        (Some($last as i64), None)
    };
    (last = $last: literal, before = $before: ident) => {
        (Some($last as i64), Some($before))
    };
}

#[tokio::test]
async fn test_paging_forward() -> anyhow::Result<()> {
    let db: Database;
    setup!(db);

    #[derive(Serialize, Deserialize, Debug, Model)]
    #[dojo(name = "users", sort_keys = ["created_at", "id"])]
    struct User {
        id: Uuid,
        name: String,
        email: String,
        created_at: NaiveDateTime,
        updated_at: NaiveDateTime,
    }

    create_users!(db, names = "linh1", "linh2", "linh3");

    let (first, after) = create_paging_args!(first = 1);
    let pagination = db.bind::<User>().cursor(first, after, None, None).await?;
    assert_that!(
        pagination.items,
        contains_each![pat!(User {
            id: anything(),
            name: eq("linh1"),
            email: eq("linh1@gmail.com"),
            created_at: anything(),
            updated_at: anything(),
        })]
    );
    assert_that!(pagination.has_next, eq(true));
    assert_that!(pagination.has_previous, eq(false));

    let cursor = pagination.end_cursor().unwrap();
    let (first, after) = create_paging_args!(first = 1, after = cursor);
    let pagination = db.bind::<User>().cursor(first, after, None, None).await?;
    assert_that!(
        pagination.items,
        contains_each![pat!(User {
            id: anything(),
            name: eq("linh2"),
            email: eq("linh2@gmail.com"),
            created_at: anything(),
            updated_at: anything(),
        })]
    );
    assert_that!(pagination.has_next, eq(true));
    assert_that!(pagination.has_previous, eq(false));

    let cursor = pagination.end_cursor().unwrap();
    let (first, after) = create_paging_args!(first = 1, after = cursor);
    let pagination = db.bind::<User>().cursor(first, after, None, None).await?;
    assert_that!(
        pagination.items,
        contains_each![pat!(User {
            id: anything(),
            name: eq("linh3"),
            email: eq("linh3@gmail.com"),
            created_at: anything(),
            updated_at: anything(),
        })]
    );
    assert_that!(pagination.has_next, eq(false));
    assert_that!(pagination.has_previous, eq(false));

    let cursor = pagination.end_cursor().unwrap();
    let (first, after) = create_paging_args!(first = 1, after = cursor);
    let pagination = db.bind::<User>().cursor(first, after, None, None).await?;
    assert_that!(pagination.items, empty());
    assert_that!(pagination.has_next, eq(false));
    assert_that!(pagination.has_previous, eq(false));

    Ok(())
}

#[tokio::test]
async fn test_paging_backward() -> anyhow::Result<()> {
    let db: Database;
    setup!(db);

    #[derive(Serialize, Deserialize, Debug, Model)]
    #[dojo(name = "users", sort_keys = ["created_at", "id"])]
    struct User {
        id: Uuid,
        name: String,
        email: String,
        created_at: NaiveDateTime,
        updated_at: NaiveDateTime,
    }

    create_users!(db, names = "linh1", "linh2", "linh3");

    let (last, before) = create_paging_args!(last = 1);
    let pagination = db.bind::<User>().cursor(None, None, last, before).await?;
    assert_that!(
        pagination.items,
        contains_each![pat!(User {
            id: anything(),
            name: eq("linh3"),
            email: eq("linh3@gmail.com"),
            created_at: anything(),
            updated_at: anything(),
        })]
    );
    assert_that!(pagination.has_next, eq(false));
    assert_that!(pagination.has_previous, eq(true));

    let cursor = pagination.end_cursor().unwrap();
    let (last, before) = create_paging_args!(last = 1, before = cursor);
    let pagination = db.bind::<User>().cursor(None, None, last, before).await?;
    assert_that!(
        pagination.items,
        contains_each![pat!(User {
            id: anything(),
            name: eq("linh2"),
            email: eq("linh2@gmail.com"),
            created_at: anything(),
            updated_at: anything(),
        })]
    );
    assert_that!(pagination.has_next, eq(false));
    assert_that!(pagination.has_previous, eq(true));

    let cursor = pagination.end_cursor().unwrap();
    let (last, before) = create_paging_args!(last = 1, before = cursor);
    let pagination = db.bind::<User>().cursor(None, None, last, before).await?;
    assert_that!(
        pagination.items,
        contains_each![pat!(User {
            id: anything(),
            name: eq("linh1"),
            email: eq("linh1@gmail.com"),
            created_at: anything(),
            updated_at: anything(),
        })]
    );
    assert_that!(pagination.has_next, eq(false));
    assert_that!(pagination.has_previous, eq(false));

    let cursor = pagination.end_cursor().unwrap();
    let (last, before) = create_paging_args!(last = 1, before = cursor);
    let pagination = db.bind::<User>().cursor(None, None, last, before).await?;
    assert_that!(pagination.items, empty());
    assert_that!(pagination.has_next, eq(false));
    assert_that!(pagination.has_previous, eq(false));

    Ok(())
}
