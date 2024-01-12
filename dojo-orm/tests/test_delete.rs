use chrono::{NaiveDateTime, Utc};
use googletest::prelude::*;
use googletest::{assert_that, pat};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use common::*;
use dojo_macros::Model;
use dojo_orm::predicates::equals;
use dojo_orm::Database;

mod common;

#[tokio::test]
async fn test_delete_1() -> anyhow::Result<()> {
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

    let id = Uuid::new_v4();
    db.insert(&User {
        id,
        name: "linh12".to_string(),
        email: "linh12@gmail.com".to_string(),
        created_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
    })
    .await?;

    let user = db
        .delete::<User>()
        .where_by(equals("id", &id))
        .exec()
        .await?;
    assert_that!(
        user,
        pat!(User {
            id: anything(),
            name: eq("linh12".to_string()),
            email: eq("linh12@gmail.com".to_string()),
            created_at: anything(),
            updated_at: anything(),
        })
    );

    Ok(())
}
