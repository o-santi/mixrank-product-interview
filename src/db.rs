use sqlx::sqlite::{SqliteConnectOptions, SqliteConnection};
use sqlx::ConnectOptions; use std::str::FromStr;

pub type SqlResult<T> = Result<T, sqlx::Error>;

pub async fn create_connection() -> SqlResult<SqliteConnection> {
  SqliteConnectOptions::from_str("sqlite://data.db")
    .expect("Invalid sqlite url")
    .connect().await
}

/// Finds the number of apps that unninstalled `from_sdk`
/// and are now using `to_sdk`.
pub async fn get_churned(conn: &mut SqliteConnection, from_sdk: &str, to_sdk: &str) -> SqlResult<i32> {
  let count: (i32,) = sqlx::query_as(r#"
    SELECT count(used_before.app_id)
    FROM (
      SELECT app_id FROM (  (
          SELECT App.id FROM (app as App
            INNER JOIN app_sdk as AppSdk ON AppSdk.app_id = App.id
            INNER JOIN sdk as Sdk ON AppSdk.sdk_id = Sdk.id
          )
          WHERE Sdk.name = $1
          AND AppSdk.installed = 1
        ) as InstalledToday
        -- selects all the app that have sdk = to_sdk installed today
        -- and from that, select all sdk's that were used by that app
        -- but are installed anymore
        INNER JOIN app_sdk as AppSdk ON AppSdk.app_id = InstalledToday.id
        INNER JOIN sdk as Sdk ON AppSdk.sdk_id = Sdk.id
      ) WHERE AppSdk.installed = 0
        AND Sdk.name = $2
    ) as used_before
  "#).bind(to_sdk).bind(from_sdk).fetch_one(conn).await?;
  Ok(count.0)
}
