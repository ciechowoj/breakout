use anyhow::{Error, Result, bail};
use std::env;
use std::io;
use std::io::Read;
use std::io::prelude::*;
use std::str::FromStr;
use std::fs::OpenOptions;
use tokio_postgres::{Client, NoTls};
use serde::{Serialize, Deserialize};
use chrono::{Utc};
use uuid::Uuid;
use apilib::*;

// GET /api/score/list -> [ { "player": "Maxymilian TheBest", "score": 1000 }, {}, ... ]
// POST /api/score/add { "player": "Maxymilian TheBest", "score": 1000 }

// POST /api/score/new (score : i64) -> uuid top 9 + id
// POST /api/score/rename (id : uuid, player : String) -> ()

// POST /api/session/new -> "<uuid>"
// POST /api/session/heartbeat -> 200 OK

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    method : String,
    uri : String,
    path : String,
    query : String,
    content : String
}

fn get_request() -> Result<Request, anyhow::Error> {
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

    let uri = env::var(REQUEST_URI)?;
    let uri_clone = uri.clone();

    let mut split_itr = uri_clone.splitn(2, '?');
    let path = split_itr.next().unwrap();

    Ok(Request {
        method: env::var(REQUEST_METHOD)?,
        uri: uri,
        path: path.to_owned(),
        query: env::var(QUERY_STRING)?,
        content: content
    })
}

async fn add_score(client : &Client, player : String, score : i64) -> Result<String, anyhow::Error> {
    client.execute(
        "CREATE TABLE IF NOT EXISTS high_scores (
            id uuid PRIMARY KEY,
            player varchar(128) NOT NULL,
            score bigint,
            created_time timestamptz);", &[]).await?;

    client.execute(
        "INSERT INTO high_scores(id, player, score, created_time)
        VALUES ($1, $2, $3, $4);", &[&Uuid::new_v4(), &player, &score, &Utc::now()]).await?;

    return Ok("{}".to_owned());
}

async fn add_score_http(client : &Client, request : &Request) -> Result<String, anyhow::Error> {
    let score : PlayerScore = serde_json::from_str(request.content.as_str())?;
    return add_score(client, score.player, score.score).await;
}

async fn get_scores_http(client : &Client) -> Result<String, anyhow::Error> {
    let rows = client
        .query("SELECT player, score FROM high_scores;", &[])
        .await?;

    let scores : Vec<PlayerScore> = rows.iter()
        .map(|row| PlayerScore { player: row.get("player"), score: row.get("score") }).collect();

    return Ok(serde_json::to_string(&scores)?);
}

fn load_connection_string() -> Result<String, anyhow::Error> {
    fn get_http_host() -> Result<String, anyhow::Error> {
        const HTTP_HOST : &'static str = "HTTP_HOST";

        let host : String = match env::var(HTTP_HOST) {
            Ok(value) => Ok(value),
            Err(env::VarError::NotPresent) => Ok("".to_owned()),
            Err(error @ env::VarError::NotUnicode(_)) => Err(error)
        }?;

        return Ok(host);
    }

    let result = include!("../connection-string.fn");

    return Ok(result.to_owned());
}

fn print_output(output : &str) {
    println!("Content-Type: application/json\n");
    println!("{}", output);
}

fn open_log_file() -> Result<std::fs::File, anyhow::Error> {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .write(true)
        .open("error.log")?;

    return Ok(file);
}

async fn inner_main() -> Result<(), anyhow::Error> {
    let connection_string = load_connection_string()?;

    let (client, connection) =
        tokio_postgres::connect(connection_string.as_ref(), NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    
    let request = get_request()?;

    match (request.method.as_str(), request.path.as_str()) {
        ("GET", "/api/score/list") => print_output(get_scores_http(&client).await?.as_str()),
        ("POST", "/api/score/add") => print_output(add_score_http(&client, &request).await?.as_str()),
        _ => print_output("{}")
    };

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let result = inner_main().await;

    return match result {
        Err(error) => {
            let mut file = open_log_file().unwrap();
            writeln!(file, "{}", error.to_string()).unwrap();
            Err(error)
        },
        Ok(()) => Ok(())
    };
}
