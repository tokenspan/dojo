## Dojo ORM

### Installation
```toml
[dependencies]
dojo-orm = { git = "https://github.com/tokenspan/dojo-orm" }
dojo-macros = { git = "https://github.com/tokenspan/dojo-macros" }
```

### Usage

#### Insert
```rust
#[derive(Serialize, Deserialize, Debug, Model)]
#[dojo(name = "users", sort_keys = ["created_at", "id"])]
struct User {
    id: Uuid,
    name: String,
    created_at: NaiveDateTime,
}

async fn run() -> anyhow::Result<()> {
    let input = User {
        id: Uuid::new_v4(),
        name: "linh".to_string(),
        created_at: Utc::now().naive_utc(),
    };
    
    let url = "";
    let db = Database::new(url).await?;
    let user = db.insert(&input).await?;
}
```

#### Insert Many
```rust
#[derive(Serialize, Deserialize, Debug, Model)]
#[dojo(name = "users", sort_keys = ["created_at", "id"])]
struct User {
    id: Uuid,
    name: String,
    created_at: NaiveDateTime,
}

async fn run() -> anyhow::Result<()> {
    let input = User {
        id: Uuid::new_v4(),
        name: "linh".to_string(),
        created_at: Utc::now().naive_utc(),
    };

    let url = "";
    let db = Database::new(url).await?;
    let user = db.insert_many::<User>(&[input]).await?;
}
```

#### Update
```rust
#[derive(Serialize, Deserialize, Debug, Model)]
#[dojo(name = "users", sort_keys = ["created_at", "id"])]
struct User {
    id: Uuid,
    name: String,
    created_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Debug, UpdateModel)]
struct UpdateUser {
    name: Option<String>,
}

async fn run() -> anyhow::Result<()> {
    let input = UpdateUser {
        name: "linh".to_string(),
    };

    let url = "";
    let db = Database::new(url).await?;
    let user = db.insert(&input).await?;

    let input = UpdateUser {
        name: Some("linh".to_string()),
    };

    let id = Uuid::new_v4();
    let user = db.update::<User, UpdateUser>(&input).where_by(equals("id", &id)).await?;
}
```

#### Delete
```rust
#[derive(Serialize, Deserialize, Debug, Model)]
#[dojo(name = "users", sort_keys = ["created_at", "id"])]
struct User {
    id: Uuid,
    name: String,
    created_at: NaiveDateTime,
}

async fn run() -> anyhow::Result<()> {
    let input = UpdateUser {
        name: "linh".to_string(),
    };

    let url = "";
    let db = Database::new(url).await?;

    let id = Uuid::new_v4();
    let user = db
        .delete::<User>()
        .where_by(equals("id", &id))
        .exec()
        .await?;
}
```

#### Select
```rust
#[derive(Serialize, Deserialize, Debug, Model)]
#[dojo(name = "users", sort_keys = ["created_at", "id"])]
struct User {
    id: Uuid,
    name: String,
    created_at: NaiveDateTime,
}

async fn run() -> anyhow::Result<()> {
    let input = UpdateUser {
        name: "linh".to_string(),
    };

    let url = "";
    let db = Database::new(url).await?;

    let id = Uuid::new_v4();
    let users = db
        .bind::<User>()
        .where_by(and(&[equals("name", &"linh1")]))
        .limit(2)
        .await?;
}
```

#### Cursor paging
```rust
#[derive(Serialize, Deserialize, Debug, Model)]
#[dojo(name = "users", sort_keys = ["created_at", "id"])]
struct User {
    id: Uuid,
    name: String,
    created_at: NaiveDateTime,
}

async fn run() -> anyhow::Result<()> {
    let input = UpdateUser {
        name: "linh".to_string(),
    };

    let url = "";
    let db = Database::new(url).await?;

    let last = Some(10);
    let before = None;
    let pagination = db.bind::<User>().cursor(None, None, last, before).await?;
}
```