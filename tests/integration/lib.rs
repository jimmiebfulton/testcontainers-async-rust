mod example_impl;

use crate::example_impl::ExampleImage;

use testcontainers_async::modules::cockroachdb::CockroachDbImage;
use testcontainers_async::modules::generic::GenericImage;
use testcontainers_async::modules::mysql::MySqlImage;
use testcontainers_async::modules::postgresql::PostgresImage;
use testcontainers_async::modules::redis::RedisImage;
use testcontainers_async::tasks::MatchLogOutput;
use testcontainers_async::{
    AdminContainer, Container, DatabaseContainer, Image, ServiceContainer, TestcontainerError,
};

fn init() {
    let _ = env_logger::builder().try_init();
}

#[tokio::test]
async fn test_generic() -> Result<(), TestcontainerError> {
    init();
    let redis = GenericImage::new("redis", "latest")
        .with_task(MatchLogOutput::containing("Ready to accept connections"))
        .start_container()
        .await?;

    let port = redis.host_port_for("6379/tcp").await?;
    println!("Redis exposed on port {port}");

    Ok(())
}

#[tokio::test]
async fn test_redis() -> Result<(), TestcontainerError> {
    init();
    let redis = RedisImage::default().start_container().await?;

    let port = redis.service_port().await?;
    println!("Redis exposed on port {port}");

    Ok(())
}

#[tokio::test]
async fn test_postgres() -> Result<(), TestcontainerError> {
    init();
    let postgres = PostgresImage::default()
        .with_database("example-service")
        .with_username("test")
        .with_password("test")
        .start_container()
        .await?;

    let connect_url = postgres.connect_url().await?;
    let connect_cli = postgres.connect_cli().await?;
    let jdbc_url = postgres.jdbc_url().await?;

    println!("Connect URL: '{connect_url}'");
    println!("Connect CLI: {connect_cli}");
    println!("JDBC URL: {jdbc_url}");
    Ok(())
}

#[tokio::test]
async fn test_cockroach() -> Result<(), TestcontainerError> {
    init();
    let crdb = CockroachDbImage::default().start_container().await?;

    let service_port = crdb.service_port().await?;
    let admin_port = crdb.admin_port().await?;

    println!("CockroachDB database exposed on port {service_port}");
    println!("CockroachDB ui exposed on port {admin_port}");

    Ok(())
}

#[tokio::test]
async fn test_mysql() -> Result<(), TestcontainerError> {
    init();
    let mysql = MySqlImage::default()
        .with_database("example-service")
        .with_username("test")
        .with_password("test")
        .start_container()
        .await?;

    let port = mysql.service_port().await?;
    println!("MySQL exposed on port {port}");
    Ok(())
}

#[tokio::test]
async fn test_example_impl() -> Result<(), TestcontainerError> {
    init();
    let example = ExampleImage::default().start_container().await?;

    let port = example.primary_port().await.expect("Port expected");
    assert!(port > 0);
    println!("Example started on port: {port}");

    Ok(())
}
