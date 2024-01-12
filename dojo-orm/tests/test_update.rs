use chrono::{NaiveDateTime, Utc};
use googletest::prelude::*;
use googletest::{assert_that, pat};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use common::*;
use dojo_macros::{EmbeddedModel, Model, UpdateModel};
use dojo_orm::predicates::equals;
use dojo_orm::Database;

mod common;

#[tokio::test]
async fn test_update_1() -> anyhow::Result<()> {
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

    #[derive(UpdateModel, Debug)]
    struct UpdateUser {
        name: Option<String>,
        email: Option<String>,
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

    let input = UpdateUser {
        name: Some("linh13".to_string()),
        email: None,
    };

    let user = db
        .update::<User, UpdateUser>(&input)
        .where_by(equals("id", &id))
        .exec()
        .await?;
    assert_that!(
        user,
        pat!(User {
            id: anything(),
            name: eq("linh13".to_string()),
            email: eq("linh12@gmail.com".to_string()),
            created_at: anything(),
            updated_at: anything(),
        })
    );

    Ok(())
}

#[tokio::test]
async fn test_update_embedded() -> anyhow::Result<()> {
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

    #[derive(UpdateModel, Debug)]
    struct UpdateProduct {
        name: Option<String>,
        detail: Option<ProductDetail>,
        #[dojo(nullable)]
        price: Option<i32>,
    }

    let id = Uuid::new_v4();
    db.insert(&Product {
        id,
        name: "product 1".to_string(),
        detail: Some(ProductDetail {
            manufacturer: "Company A".to_string(),
        }),
        price: Some(100),
        created_at: Utc::now().naive_utc(),
    })
    .await?;

    let product = db
        .update::<Product, UpdateProduct>(&UpdateProduct {
            name: Some("product 2".to_string()),
            detail: Some(ProductDetail {
                manufacturer: "Company B".to_string(),
            }),
            price: None,
        })
        .where_by(equals("id", &id))
        .exec()
        .await?;

    assert_that!(
        product,
        pat!(Product {
            id: anything(),
            name: eq("product 2".to_string()),
            detail: some(pat!(ProductDetail {
                manufacturer: eq("Company B".to_string()),
            })),
            price: none(),
            created_at: anything(),
        })
    );

    Ok(())
}
