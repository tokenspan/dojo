use chrono::{NaiveDateTime, Utc};
use common::*;
use dojo_macros::Model;
use dojo_orm::predicates::*;
use dojo_orm::Database;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

mod common;

#[tokio::test]
async fn test_raw_text_search() -> anyhow::Result<()> {
    let db: Database;
    setup!(db);

    #[derive(Serialize, Deserialize, Debug, Model)]
    #[dojo(name = "movies", sort_keys = ["created_at", "id"])]
    struct Movie {
        id: Uuid,
        name: String,
        detail: String,
        created_at: NaiveDateTime,
    }

    let user1 = Movie {
        id: Uuid::new_v4(),
        name: "The Shawshank Redemption".to_string(),
        detail: "The Shawshank Redemption is a 1994 American prison drama film written and directed by Frank Darabont, based on the 1982 Stephen King novella Rita Hayworth".to_string(),
        created_at: Utc::now().naive_utc(),
    };
    let user2 = Movie {
        id: Uuid::new_v4(),
        name: "The Godfather".to_string(),
        detail: "The son of the patriarch of the most powerful Mafia clan in New York returns home from war determined to live his own life - but is forced to take up arms".to_string(),
        created_at: Utc::now().naive_utc(),
    };

    db.insert(&[&user1, &user2]).all().await?;

    let user = db
        .bind::<Movie>()
        .where_by(raw_str(
            "detail @@ websearch_to_tsquery('english', 'most powerful Mafia')",
        ))
        .first()
        .await?;

    println!("user: {:?}", user);

    Ok(())
}

#[tokio::test]
async fn test_text_search() -> anyhow::Result<()> {
    let db: Database;
    setup!(db);

    #[derive(Serialize, Deserialize, Debug, Model)]
    #[dojo(name = "movies", sort_keys = ["created_at", "id"])]
    struct Movie {
        id: Uuid,
        name: String,
        detail: String,
        created_at: NaiveDateTime,
    }

    let user1 = Movie {
        id: Uuid::new_v4(),
        name: "The Shawshank Redemption".to_string(),
        detail: "The Shawshank Redemption is a 1994 American prison drama film written and directed by Frank Darabont, based on the 1982 Stephen King novella Rita Hayworth".to_string(),
        created_at: Utc::now().naive_utc(),
    };
    let user2 = Movie {
        id: Uuid::new_v4(),
        name: "The Godfather".to_string(),
        detail: "The son of the patriarch of the most powerful Mafia clan in New York returns home from war determined to live his own life - but is forced to take up arms".to_string(),
        created_at: Utc::now().naive_utc(),
    };

    db.insert(&[&user1, &user2]).all().await?;

    let user = db
        .bind::<Movie>()
        .where_by(text_search("detail", "english", "powerful Mafia clan"))
        .first()
        .await?;

    println!("user: {:?}", user);

    Ok(())
}
