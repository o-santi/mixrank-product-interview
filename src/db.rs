use cfg_if::cfg_if;
use leptos::*;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct App {
  id: i32,
  name: String,
  company_url: String,
  release_date: String,
  genre_id: i32,
  artwork_large_url: String,
  seller_name: String,
  five_star_ratings: i32,
  four_star_ratings: i32,
  three_star_ratings: i32,
  two_star_ratings: i32,
  one_star_ratings: i32
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Sdk {
  pub id: i32,
  pub name: String,
  pub slug: String,
  pub url: String,
  pub description: String
}


cfg_if! {
  if #[cfg(feature = "ssr")] {
    
    use sqlx::sqlite::{SqliteConnectOptions, SqliteConnection};
    use sqlx::ConnectOptions;
    use std::str::FromStr;
    
    pub type SqlResult<T> = Result<T, sqlx::Error>;
    
    
    pub async fn create_connection() -> SqlResult<SqliteConnection> {
      SqliteConnectOptions::from_str("sqlite://data.db")
        .expect("Invalid database string.") // should be a hard error.
        .connect().await
    }
  
    pub fn register_server_functions() {
      _ = GetAllChurned::register();
    }
    
    /// Finds the number of apps that unninstalled `from_sdk`
    /// and are now using `to_sdk`.
    pub async fn get_churned(conn: &mut SqliteConnection, from_sdk: &str, to_sdk: &str) -> SqlResult<Vec<App>> {
      use sqlx::Row;
      let apps = sqlx::query(r#"
    SELECT *
    FROM (
        (
          SELECT App.id FROM (app as App
            INNER JOIN app_sdk as AppSdk ON AppSdk.app_id = App.id
            INNER JOIN sdk as Sdk ON AppSdk.sdk_id = Sdk.id
          )
          WHERE Sdk.name = $1
          AND AppSdk.installed = 1
        ) as InstalledToday
        -- selects all the apps that have sdk = to_sdk installed today
        -- and from that, select all sdk's that were used by that app
        -- but are not installed anymore
        INNER JOIN app_sdk as AppSdk ON AppSdk.app_id = InstalledToday.id
        INNER JOIN sdk as Sdk ON AppSdk.sdk_id = Sdk.id
        INNER JOIN app as App ON AppSdk.app_id = App.id
      ) WHERE AppSdk.installed = 0
        AND Sdk.name = $2
     "#)
        .bind(to_sdk)
        .bind(from_sdk)
        .fetch_all(conn)
        .await?
        .iter()
        .map(|row| App {
          id: row.get("app_id"),
          name: row.get("name"),
          company_url: row.get("company_url"),
          release_date: row.get("release_date"),
          genre_id: row.get("genre_id"),
          artwork_large_url: row.get("artwork_large_url"),
          seller_name: row.get("seller_name"),
          five_star_ratings: row.get("five_star_ratings"),
          four_star_ratings: row.get("four_star_ratings"),
          three_star_ratings: row.get("three_star_ratings"),
          two_star_ratings: row.get("two_star_ratings"),
          one_star_ratings: row.get("one_star_ratings")
        }).collect();
      Ok(apps)
    }
  }
}

#[server(GetAllSdks)]
pub async fn get_all_sdks() -> Result<Vec<Sdk>, ServerFnError> {
  use sqlx::Row;
  let mut conn = create_connection().await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;
  let sdks = sqlx::query("SELECT * FROM Sdk")
    .fetch_all(&mut conn).await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?
    .iter()
    .map(|row| {
      Sdk {
        id: row.get("id"),
        name: row.get("name"),
        slug: row.get("slug"),
        url: row.get("url"),
        description: row.get("description")
      }
    })
    .collect();
  Ok(sdks)
}

#[server(GetAllChurned, "/api")]
pub async fn get_all_churned(sdks: Vec<String>) -> Result<Vec<(String, Vec<Vec<App>>)>, ServerFnError> {
  let mut conn = create_connection().await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;
  let mut ret = Vec::with_capacity(sdks.len());
  for from in sdks.iter() {
    let mut row = Vec::with_capacity(sdks.len());
    for to in sdks.iter() {
      let churned_apps = get_churned(&mut conn, from, to).await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;
      row.push(churned_apps);
    }
    ret.push((from.clone(), row));
  }
  Ok(ret)
}

