use std::collections::HashMap;

use cfg_if::cfg_if;
use leptos::*;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct App {
  pub id: i32,
  pub name: String,
  pub company_url: String,
  pub release_date: String,
  pub genre_id: i32,
  pub artwork_large_url: String,
  pub seller_name: String,
  pub five_star_ratings: i32,
  pub four_star_ratings: i32,
  pub three_star_ratings: i32,
  pub two_star_ratings: i32,
  pub one_star_ratings: i32
}

impl App {
  pub fn rating(&self) -> f32 {
    (self.one_star_ratings       +
     self.two_star_ratings   * 2 +
     self.three_star_ratings * 3 +
     self.four_star_ratings  * 4 +
     self.five_star_ratings  * 5) as f32 / self.rating_count() as f32
  }

  pub fn rating_count(&self) -> i32 {
    self.one_star_ratings    +
     self.two_star_ratings   +
     self.three_star_ratings +
     self.four_star_ratings  +
     self.five_star_ratings
  }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Sdk {
  pub id: i32,
  pub name: String,
  pub slug: String,
  pub url: String,
  pub description: String
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub enum Column {
  Sdk(Sdk),
  All
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

    pub async fn get_all_apps(conn: &mut SqliteConnection) -> SqlResult<i32> {
      let (count,): (i32,) = sqlx::query_as("SELECT count(*) from app;").fetch_one(conn).await?;
      Ok(count)
    }
    
    /// Finds the number of apps that unninstalled `from_sdk`
    /// and are now using `to_sdk`.
    pub async fn get_churned(conn: &mut SqliteConnection, from_sdk: &Column, to_sdk: &Column) -> SqlResult<Vec<App>> {
      use sqlx::Row;

      let equals_from = match from_sdk {
        Column::Sdk(sdk) => format!("Sdk.id = {}", sdk.id),
        Column::All => "1".into(),
      };
      let equals_to = match to_sdk {
        Column::Sdk(sdk) => format!("Sdk.id = {}", sdk.id),
        Column::All => "1".into(),
      };
      
      let query = if from_sdk == to_sdk {
        format!("SELECT DISTINCT AppSdk.app_id, App.* FROM (app as App
                   INNER JOIN app_sdk as AppSdk ON AppSdk.app_id = App.id
                   INNER JOIN sdk as Sdk ON AppSdk.sdk_id = Sdk.id)
                 WHERE {equals_from} AND AppSdk.installed = 1")
      } else {
        format!("
         SELECT *
           FROM (
          (
            SELECT App.id FROM (app as App
              INNER JOIN app_sdk as AppSdk ON AppSdk.app_id = App.id
              INNER JOIN sdk as Sdk ON AppSdk.sdk_id = Sdk.id
            )
            WHERE {equals_to}
            AND AppSdk.installed = 1
          ) as InstalledToday
          -- selects all the apps that have sdk = to_sdk installed today
          -- and from that, select all sdk's that were used by that app
          -- but are not installed anymore
          INNER JOIN app_sdk as AppSdk ON AppSdk.app_id = InstalledToday.id
          INNER JOIN sdk as Sdk ON AppSdk.sdk_id = Sdk.id
          INNER JOIN app as App ON AppSdk.app_id = App.id
        ) WHERE AppSdk.installed = 0
          AND {equals_from}
       ")
      };
      let apps = sqlx::query(&query).fetch_all(conn)
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

#[server(GetAllChurned, "/api", "Cbor")]
pub async fn get_all_churned(sdks: Vec<Column>) -> Result<HashMap<(Column, Column), Vec<App>>, ServerFnError> {
  let mut conn = create_connection().await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;
  let mut map = HashMap::with_capacity(sdks.len() * sdks.len());
  for from in sdks.iter() {
    for to in sdks.iter() {
      let churned_apps = get_churned(&mut conn, from, to).await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;
      map.insert((from.clone(), to.clone()), churned_apps);
    }
  }
  Ok(map)
}

#[server(GetTotalAppsCount)]
pub async fn get_total_apps_count() -> Result<i32, ServerFnError> {
  let mut conn = create_connection().await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;
  let count = get_all_apps(&mut conn).await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;
  Ok(count)
}
