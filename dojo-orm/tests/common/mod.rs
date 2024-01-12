pub mod embedded {
    use refinery::embed_migrations;

    embed_migrations!("./tests/migrations");
}

#[macro_export]
macro_rules! setup {
    ($db: ident) => {
        tracing_subscriber::fmt().init();
        let docker = testcontainers_modules::testcontainers::clients::Cli::default();
        let node = docker.run(testcontainers_modules::postgres::Postgres::default());
        let url = &format!(
            "postgres://postgres:postgres@localhost:{}/postgres",
            node.get_host_port_ipv4(5432)
        );

        $db = Database::new(url).await?;

        let mut conn = $db.get().await?;
        use std::ops::DerefMut;
        let client = conn.deref_mut();
        embedded::migrations::runner()
            .run_async(client)
            .await
            .unwrap();
    };
}
