use anyhow::*;
use std::env;
use std::io;
use std::io::Read;
use std::io::prelude::*;
use std::str::FromStr;
use std::fs::OpenOptions;
use tokio_postgres::{Client, NoTls};
use serde::{de};
use chrono::{Utc};
use uuid::Uuid;
use apilib::*;
use http::{Request, Response, StatusCode};

// GET /api/score/list -> [ { "player": "Maxymilian TheBest", "score": 1000 }, {}, ... ]
// POST /api/score/add { "player": "Maxymilian TheBest", "score": 1000 }

// POST /api/score/new (score : i64) -> uuid top 9 + id
// POST /api/score/rename (id : uuid, player : String) -> ()

// POST /api/session/new -> "<uuid>"
// POST /api/session/heartbeat -> 200 OK

/*#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    method : String,
    uri : String,
    path : String,
    query : String,
    content : String
}*/

fn get_request() -> anyhow::Result<Request<String>> {
    const CONTENT_LENGTH : &'static str = "CONTENT_LENGTH";
    const REQUEST_METHOD : &'static str = "REQUEST_METHOD";
    const REQUEST_URI : &'static str = "REQUEST_URI";

    let mut content = String::new();

    let content_length : usize = match env::var(CONTENT_LENGTH) {
        Ok(value) => Ok(usize::from_str(value.as_ref())?),
        Err(env::VarError::NotPresent) => Ok(0),
        Err(error @ env::VarError::NotUnicode(_)) => Err(error)
    }?;

    if content_length != 0 {
        io::stdin().read_to_string(&mut content)?;
    }

    let request = Request::builder()
        .method(env::var(REQUEST_METHOD)?.as_str())
        .uri(env::var(REQUEST_URI)?)
        .body(content)?;
        
    return Ok(request);
}

async fn add_score(client : &Client, request : &Request<AddScoreRequest>) -> anyhow::Result<Response<()>> {
    client.execute(
        "CREATE TABLE IF NOT EXISTS high_scores (
            id uuid PRIMARY KEY,
            name varchar(128) NOT NULL,
            score bigint,
            created_time timestamptz);", &[]).await?;

    let body = &request.body();

    client.execute(
        "INSERT INTO high_scores(id, name, score, created_time)
        VALUES ($1, $2, $3, $4);", &[&Uuid::new_v4(), &body.name, &body.score, &Utc::now()]).await?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .body(())?;

    return Ok(response);
}

async fn list_scores_http(client : &Client, request : &Request<ListScoresRequest>) -> anyhow::Result<Response<Vec<PlayerScore>>> {
    let body = request.body();

    let rows = if let Some(limit) = body.limit {
        client
            .query("SELECT ROW_NUMBER() OVER (ORDER BY score DESC), name, score 
                    FROM high_scores
                    ORDER BY score DESC
                    LIMIT $1;", &[&limit])
            .await?
    }
    else {
        client
            .query("SELECT ROW_NUMBER() OVER (ORDER BY score DESC), name, score 
                    FROM high_scores
                    ORDER BY score DESC;", &[])
            .await?
    };

    let scores : Vec<PlayerScore> = rows.iter()
        .map(|row| PlayerScore { index: row.get::<&str, i64>("row_number") - 1, name: row.get("name"), score: row.get("score") })
        .collect();

    let response = Response::builder()
        .status(StatusCode::OK)
        .body(scores)?;

    return Ok(response);
}

async fn new_score(client : &Client, request : &Request<NewScoreRequest>) -> anyhow::Result<Response<NewScoreResponse>> {
    client.execute(
        "CREATE TABLE IF NOT EXISTS high_scores (
            id uuid PRIMARY KEY,
            name varchar(128) NOT NULL,
            score bigint,
            created_time timestamptz);", &[]).await?;

    let body = &request.body();

    let id = Uuid::new_v4();

    client.execute(
        "INSERT INTO high_scores(id, name, score, created_time)
        VALUES ($1, $2, $3, $4);", &[&id, &"", &body.score, &Utc::now()]).await?;

    let rows = client
        .query("SELECT ROW_NUMBER() OVER (ORDER BY score DESC), name, score 
                FROM high_scores
                ORDER BY score DESC
                LIMIT $1;", &[&body.limit])
        .await?;

    let scores : Vec<PlayerScore> = rows.iter()
        .map(|row| PlayerScore { index: row.get::<&str, i64>("row_number") - 1, name: row.get("name"), score: row.get("score") })
        .collect();

    let response = Response::builder()
        .status(StatusCode::OK)
        .body(NewScoreResponse { id: id, scores: scores })?;

    return Ok(response);
}

async fn rename_score_http(client : &Client, request : &Request<RenameScoreRequest>) -> anyhow::Result<Response<()>> {
    let body = &request.body();

    client.execute(
        "UPDATE high_scores
            SET name = $1
            WHERE id = $2;", &[&body.name, &body.id]).await?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .body(())?;

    return Ok(response);
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

    fn get_database_name() -> Result<Option<String>> {
        const DATABASE_NAME : &'static str = "DATABASE_NAME";

        let database_name : Option<String> = match env::var(DATABASE_NAME) {
            Ok(value) => Ok(Some(value)),
            Err(env::VarError::NotPresent) => Ok(None),
            Err(error @ env::VarError::NotUnicode(_)) => Err(error)
        }?;

        return Ok(database_name);
    }

    let result = include!("../connection-string.fn").to_owned();

    let result = if let Some(database_name) = get_database_name()? {
        result.replace("{}", database_name.as_str())
    }
    else {
        result
    };

    return Ok(result);
}

fn print_output<T : serde::Serialize>(response : &Response<T>) -> anyhow::Result<()> {
    println!("Content-Type: application/json\n");
    println!("{}", serde_json::to_string(&response.body())?);

    return Ok(());
}

fn open_log_file() -> Result<std::fs::File, anyhow::Error> {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .write(true)
        .open("error.log")?;

    return Ok(file);
}

fn deserialize<T>(request : Request<String>) -> anyhow::Result<Request<T>>
    where for<'de> T: de::Deserialize<'de>
{
    let (parts, body) = request.into_parts();
    let body = serde_json::from_str(&body)?;
    Ok(Request::from_parts(parts, body))
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

    match (request.method().as_str(), request.uri().path()) {
        ("GET", "/api/score/list") => print_output(&list_scores_http(&client, &deserialize(request)?).await?)?,
        ("POST", "/api/score/add") => print_output(&add_score(&client, &deserialize(request)?).await?)?,
        ("POST", "/api/score/new") => print_output(&new_score(&client, &deserialize(request)?).await?)?,
        ("POST", "/api/score/rename") => print_output(&rename_score_http(&client, &deserialize(request)?).await?)?,
        _ => print_output(&Response::builder().status(StatusCode::NOT_FOUND).body(())?)?
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
