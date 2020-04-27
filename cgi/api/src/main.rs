use std::env;
use std::error;
use std::io;
use std::io::Read;
use std::str::FromStr;
use tokio_postgres::{Client, NoTls, Error};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, TimeZone, NaiveDateTime, Utc};
use uuid::Uuid;

// GET score/list -> [ { "player": "Maxymilian TheBest", "score": 1000 }, {}, ... ]
// POST score/add { "player": "Maxymilian TheBest", "score": 1000 }

// POST session/new -> "<uuid>"
// POST session/heartbeat -> 200 OK

#[derive(Serialize, Deserialize)]
pub struct PlayerScore {
    pub player : String,
    pub score : u64
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    method : String,
    uri : String,
    query : String,
    content : String
}

fn get_request() -> Result<Request, Box<dyn error::Error>> {
    const CONTENT_LENGTH : &'static str = "CONTENT_LENGTH";
    const REQUEST_METHOD : &'static str = "REQUEST_METHOD";
    const REQUEST_URI : &'static str = "REQUEST_URI";
    const QUERY_STRING : &'static str = "QUERY_STRING";

    let mut content = String::new();

    let content_length : usize = match env::var(CONTENT_LENGTH) {
        Ok(value) => Ok(usize::from_str(value.as_ref())?),
        Err(env::VarError::NotPresent) => Ok(0),
        Err(error @ env::VarError::NotUnicode(_)) => Err(error)
    }?;

    if content_length != 0 {
        io::stdin().read_to_string(&mut content)?;
    }

    Ok(Request {
        method: env::var(REQUEST_METHOD)?,
        uri: env::var(REQUEST_URI)?,
        query: env::var(QUERY_STRING)?,
        content: content
    })
}

async fn add_score(client : &Client, player : String, score : i64) -> Result<(), Box<dyn error::Error>> {
    client.execute(
        "CREATE TABLE IF NOT EXISTS high_scores (
            id uuid PRIMARY KEY,
            player varchar(128) NOT NULL,
            score bigint,
            created_time timestamptz);", &[]).await?;

    client.execute(
        "INSERT INTO high_scores(id, player, score, created_time)
        VALUES ($1, $2, $3, $4);", &[&Uuid::new_v4(), &player, &score, &Utc::now()]).await?;

    Ok(())
}

async fn get_scores(client : &Client) -> Result<Vec<PlayerScore>, Box<dyn error::Error>> {
    

}


#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let (client, connection) =
        tokio_postgres::connect("host=localhost user=wojciech dbname=wojciech password=password", NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let rows = client
        .query("SELECT $1::TEXT", &[&"hello world"])
        .await?;

    let value: &str = rows[0].get(0);
    assert_eq!(value, "hello world");

    add_score(&client, "Maxymilain Debeściak".to_owned(), 1024)
        .await?;

    let serialized = serde_json::to_string(&get_request()?)?;

    println!("Content-type: application/json\n");
    println!("{}", serialized);

    Ok(())
}



