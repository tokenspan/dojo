use chrono::{NaiveDateTime, Utc};
use googletest::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use common::*;
use dojo_macros::{EmbeddedModel, Model};
use dojo_orm::predicates::{and, equals, in_list};
use dojo_orm::Database;

mod common;

#[tokio::test]
async fn test_insert() -> anyhow::Result<()> {
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

    let input = User {
        id: Uuid::new_v4(),
        name: "linh12".to_string(),
        email: "linh12@gmail.com".to_string(),
        created_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
    };

    let user = db.insert(&input).await?;
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

    let user = db
        .bind::<User>()
        .where_by(and(&[equals("id", &user.id)]))
        .first()
        .await?;
    assert_that!(
        user,
        some(pat!(User {
            id: anything(),
            name: eq("linh12".to_string()),
            email: eq("linh12@gmail.com".to_string()),
            created_at: anything(),
            updated_at: anything(),
        }))
    );

    Ok(())
}

#[tokio::test]
async fn test_insert_many() -> anyhow::Result<()> {
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

    let input1 = User {
        id: Uuid::new_v4(),
        name: "linh12".to_string(),
        email: "linh12@gmail.com".to_string(),
        created_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
    };
    let input2 = User {
        id: Uuid::new_v4(),
        name: "linh13".to_string(),
        email: "linh13@gmail.com".to_string(),
        created_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
    };

    let users = db.insert_many(&[input1, input2]).await?;
    assert_that!(
        users,
        contains_each![
            pat!(User {
                id: anything(),
                name: eq("linh12".to_string()),
                email: eq("linh12@gmail.com".to_string()),
                created_at: anything(),
                updated_at: anything(),
            }),
            pat!(User {
                id: anything(),
                name: eq("linh13".to_string()),
                email: eq("linh13@gmail.com".to_string()),
                created_at: anything(),
                updated_at: anything(),
            })
        ]
    );

    let users = db
        .bind::<User>()
        .where_by(and(&[in_list("id", &[&users[0].id, &users[1].id])]))
        .limit(2)
        .await?;
    assert_that!(
        users,
        contains_each![
            pat!(User {
                id: anything(),
                name: eq("linh12".to_string()),
                email: eq("linh12@gmail.com".to_string()),
                created_at: anything(),
                updated_at: anything(),
            }),
            pat!(User {
                id: anything(),
                name: eq("linh13".to_string()),
                email: eq("linh13@gmail.com".to_string()),
                created_at: anything(),
                updated_at: anything(),
            })
        ]
    );

    Ok(())
}

#[tokio::test]
async fn test_insert_embedded() -> anyhow::Result<()> {
    let db: Database;
    setup!(db);

    #[derive(Serialize, Deserialize, Debug, EmbeddedModel)]
    struct ProductDetail {
        manufacturer: String,
    }

    #[derive(Serialize, Deserialize, Debug, Model)]
    #[dojo(name = "products", sort_keys = ["created_at", "id"])]
    struct Product {
        id: Uuid,
        name: String,
        detail: Option<ProductDetail>,
        price: Option<i32>,
        created_at: NaiveDateTime,
    }

    let input = Product {
        id: Uuid::new_v4(),
        name: "product 1".to_string(),
        detail: Some(ProductDetail {
            manufacturer: "Company A".to_string(),
        }),
        price: Some(100),
        created_at: Utc::now().naive_utc(),
    };

    let product = db.insert(&input).await?;
    assert_that!(
        product,
        pat!(Product {
            id: anything(),
            name: eq("product 1".to_string()),
            detail: some(pat!(ProductDetail {
                manufacturer: eq("Company A".to_string()),
            })),
            price: some(eq(100)),
            created_at: anything(),
        })
    );

    Ok(())
}
