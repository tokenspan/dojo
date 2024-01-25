use googletest::prelude::*;
use pgvector::Vector;

use common::*;
use dojo_macros::Model;
use dojo_orm::prelude::nearest;
use dojo_orm::Database;

mod common;

#[tokio::test]
async fn test_vector_search() -> anyhow::Result<()> {
    let db: Database;
    setup!(db);

    #[derive(Debug, Model)]
    #[dojo(name = "items")]
    struct Item {
        embedding: Vector,
    }

    let item1 = Item {
        embedding: Vector::from(vec![1.0, 2.0, 3.0]),
    };
    let item2 = Item {
        embedding: Vector::from(vec![4.0, 5.0, 6.0]),
    };

    let _items = db.insert(&[&item1, &item2]).all().await?;
    // assert_that!(
    //     &items,
    //     contains_each![
    //         pat!(Item {
    //             embedding: eq(Vector::from(vec![1.0, 2.0, 3.0]))
    //         }),
    //         pat!(Item {
    //             embedding: eq(Vector::from(vec![4.0, 5.0, 6.0]))
    //         })
    //     ]
    // );

    let embedding = Vector::from(vec![1.0, 2.0, 3.0]);
    let _items = db
        .bind::<Item>()
        .order_by(nearest("embedding", &embedding))
        .all()
        .await?;
    // assert_that!(
    //     &items,
    //     contains_each![
    //         pat!(Item {
    //             embedding: eq(Vector::from(vec![1.0, 2.0, 3.0]))
    //         }),
    //         pat!(Item {
    //             embedding: eq(Vector::from(vec![4.0, 5.0, 6.0]))
    //         })
    //     ]
    // );

    Ok(())
}
