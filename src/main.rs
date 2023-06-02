use product_eng_interview::db::{create_connection, get_churned, SqlResult};

#[tokio::main]
async fn main() -> SqlResult<()> {
  let mut conn = create_connection().await?;
  let apps: Vec<(String, String)> = sqlx::query_as("
   SELECT A.name, B.name FROM
     sdk as A CROSS JOIN sdk as B
  ").fetch_all(&mut conn).await?;
  let mut sum = 0;
  for (from, to) in apps {
    let apps = get_churned(&mut conn, &from, &to).await?;
    sum += apps;
    println!("{to} churned {apps} apps from {from}");
  }
  println!("{sum} total apps");
  Ok(())
}
