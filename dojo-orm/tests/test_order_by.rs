use chrono::{NaiveDateTime, Utc};
use googletest::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use common::*;
use dojo_macros::Model;
use dojo_orm::prelude::*;
use dojo_orm::Database;

mod common;

macro_rules! create_users {
    ($db: ident, names = $($name:literal),+) => {
        $db.insert_many(&[
            $(User {
                id: Uuid::new_v4(),
                name: $name.to_string(),
                email: concat!($name, "@gmail.com").to_string(),
                created_at: Utc::now().naive_utc(),
                updated_at: Utc::now().naive_utc(),
            }),+
        ]).await?;
    };
}

#[tokio::test]
async fn test_order_by_desc() -> anyhow::Result<()> {
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

    let users = db
        .bind::<User>()
        .order_by(desc("created_at"))
        .limit(2)
        .await?;

    assert_that!(
        users,
        contains_each![
            pat!(User {
                id: anything(),
                name: eq("linh3".to_string()),
                email: eq("linh3@gmail.com".to_string()),
                created_at: anything(),
                updated_at: anything(),
            }),
            pat!(User {
                id: anything(),
                name: eq("linh2".to_string()),
                email: eq("linh2@gmail.com".to_string()),
                created_at: anything(),
                updated_at: anything(),
            }),
        ]
    );

    Ok(())
}

#[tokio::test]
async fn test_order_by_asc() -> anyhow::Result<()> {
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

    let users = db
        .bind::<User>()
        .order_by(asc("created_at"))
        .order_by(desc("id"))
        .limit(2)
        .await?;

    assert_that!(
        users,
        contains_each![
            pat!(User {
                id: anything(),
                name: eq("linh1".to_string()),
                email: eq("linh1@gmail.com".to_string()),
                created_at: anything(),
                updated_at: anything(),
            }),
            pat!(User {
                id: anything(),
                name: eq("linh2".to_string()),
                email: eq("linh2@gmail.com".to_string()),
                created_at: anything(),
                updated_at: anything(),
            }),
        ]
    );

    Ok(())
}
