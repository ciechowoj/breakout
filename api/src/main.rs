use anyhow::*;
use std::env;
use std::io;
use std::io::Read;
use std::io::prelude::*;
use std::str::FromStr;
use std::str;
use serde::{de};
use serde::{Deserialize};
use chrono::{Utc};
use uuid::Uuid;
use apilib::*;
use sha2::{Sha256, Digest};
use http::{Request, Response, StatusCode};
use http::header::*;
use hex;
use base64;
use rand::prelude::*;

fn get_request() -> anyhow::Result<Request<String>> {
    const CONTENT_LENGTH : &'static str = "CONTENT_LENGTH";
    const HTTP_AUTHORIZATION : &'static str = "HTTP_AUTHORIZATION";
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

    let builder = Request::builder()
        .method(env::var(REQUEST_METHOD)?.as_str())
        .uri(env::var(REQUEST_URI)?);

    let builder = match env::var(HTTP_AUTHORIZATION) {
        Ok(value) => Ok(builder.header(AUTHORIZATION, value.as_str())),
        Err(env::VarError::NotPresent) => Ok(builder),
        Err(error @ env::VarError::NotUnicode(_)) => Err(error)
    }?;

    let request = builder
        .body(content)?;

    return Ok(request);
}

fn get_session_id_salt() -> anyhow::Result<[u8; 32]> {
    const SESSION_ID_SALT : &'static str = "SESSION_ID_SALT";

    let session_id_salt : String = match env::var(SESSION_ID_SALT) {
        Ok(value) => Ok(value),
        Err(error) => Err(error)
    }?;

    let mut bytes = [0u8; 32];
    hex::decode_to_slice(session_id_salt.as_str(), &mut bytes)?;

    return Ok(bytes);
}

async fn list_scores_http(request : &Request<ListScoresRequest>) -> anyhow::Result<Response<Vec<PlayerScore>>> {
    use simple_postgres::*;

    let body = request.body();

    let connection = Connection::new(&load_connection_string()?);

    #[derive(Debug, Deserialize)]
    struct Row {
        pub row_number : i64,
        pub name : String,
        pub score : i64
    }

    let result : std::result::Result<Vec<Row>, simple_postgres::Error> = if let Some(limit) = body.limit {
        query!(
            connection,
            "SELECT ROW_NUMBER() OVER (ORDER BY score DESC), name, score
            FROM high_scores
            ORDER BY score DESC
            LIMIT $1;",
            limit)
    }
    else {
        query!(
            connection,
            "SELECT ROW_NUMBER() OVER (ORDER BY score DESC), name, score
            FROM high_scores
            ORDER BY score DESC;")
    };

    let scores = match result {
        Err(error) => {
            if error.sql_state == SqlState::UndefinedTable {
                Ok(Vec::<PlayerScore>::new())
            }
            else {
                Err(error)
            }
        },
        Ok(rows) => {
            let scores = rows.iter()
                .map(|row| PlayerScore {
                    index: row.row_number - 1,
                    name: row.name.clone(),
                    score: row.score
                })
                .collect();
            Ok(scores)
        }
    }?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .body(scores)?;

    return Ok(response);
}

async fn new_session_id() -> anyhow::Result<Response<String>> {
    let session_id_salt = get_session_id_salt()?;
    let mut rng = thread_rng();
    let nonce = rand128(&mut rng);

    let sha256 = Sha256::new()
        .chain(session_id_salt)
        .chain(nonce)
        .finalize();

    let mut session_id = [0u8; 32];

    for i in 0..16 {
        session_id[i] = nonce[i];
        session_id[i + 16] = sha256[i];
    }

    let response = Response::builder()
        .status(StatusCode::OK)
        .body(hex::encode_upper(session_id))?;

    return Ok(response);
}

fn validate_session_id(session_id : [u8; 32]) -> anyhow::Result<bool> {
    let session_id_salt = get_session_id_salt()?;

    let mut nonce = [0u8; 16];

    for i in 0..16 {
        nonce[i] = session_id[i];
    }

    let sha256 = Sha256::new()
        .chain(session_id_salt)
        .chain(nonce)
        .finalize();

    for i in 0..16 {
        if sha256[i] != session_id[i + 16] {
            return Ok(false);
        }
    }

    return Ok(true);
}

async fn create_high_scores_table(connection : &simple_postgres::Connection) -> anyhow::Result<()> {
    use simple_postgres::*;

    query!(
        connection,
        "CREATE TABLE IF NOT EXISTS high_scores (
            id uuid PRIMARY KEY,
            name varchar(128) NOT NULL,
            score bigint,
            created_time timestamptz);")?;

    return Ok(());
}

async fn create_default_high_scores(connection : &simple_postgres::Connection) -> anyhow::Result<()> {
    use simple_postgres::*;

    create_high_scores_table(connection).await?;

    let sql = "INSERT INTO high_scores(id, name, score, created_time) VALUES ($1, $2, $3, $4);";

    query!(connection, sql, Uuid::new_v4(), "Alistair", 16000i64, Utc::now())?;
    query!(connection, sql, Uuid::new_v4(), "Ferris", 8000i64, Utc::now())?;
    query!(connection, sql, Uuid::new_v4(), "Gordon", 4000i64, Utc::now())?;
    query!(connection, sql, Uuid::new_v4(), "Henry", 2000i64, Utc::now())?;
    query!(connection, sql, Uuid::new_v4(), "Voytech", 1000i64, Utc::now())?;
    query!(connection, sql, Uuid::new_v4(), "Voytech", 1000i64, Utc::now())?;

    return Ok(());
}

async fn new_score_http(request : &Request<NewScoreRequest>) -> anyhow::Result<Response<NewScoreResponse>> {
    use simple_postgres::*;

    let body = &request.body();

    let mut decoded_session_id = [0u8; 32];
    hex::decode_to_slice(body.session_id.as_str(), &mut decoded_session_id)?;

    if !validate_session_id(decoded_session_id)? {
        let response = Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(NewScoreResponse::Error("Invalid session id!".to_owned()))?;

        return Ok(response);
    }

    let mut decoded_proof_of_work = [0u8; 32];
    hex::decode_to_slice(body.proof_of_work.as_str(), &mut decoded_proof_of_work)?;

    if !validate_proof_of_work(decoded_session_id, decoded_proof_of_work, 8).0 {
        let response = Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(NewScoreResponse::Error("Invalid proof of work!".to_owned()))?;

        return Ok(response);
    }

    let id = Uuid::from_slice(&decoded_session_id[16..])?;

    let utc_now = Utc::now();

    let connection = Connection::new(&load_connection_string()?);

    let result : std::result::Result<(), simple_postgres::Error>  = query!(
        connection,
        "INSERT INTO high_scores(id, name, score, created_time) VALUES ($1, $2, $3, $4);",
        id,
        "",
        body.score,
        utc_now);

    if let Err(error) = result {
        if error.sql_state == simple_postgres::SqlState::UniqueViolation {
            let response = Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(NewScoreResponse::Error("Session id cannot be reused!".to_owned()))?;

            return Ok(response);
        }
    }

    #[derive(Debug, Deserialize)]
    struct Row {
        pub index : i64,
        pub id : Uuid,
        pub name : String,
        pub score : i64
    }

    let rows : Vec<Row> = query!(
        connection,
        "SELECT * FROM select_adjacent_scores($1, $2);",
         id,
         body.limit)?;

    let mut scores = Vec::<PlayerScore>::new();
    let mut index = -1i64;

    for row in rows {
        scores.push(PlayerScore {
            index: row.index - 1i64,
            name: row.name,
            score: row.score
        });

        if row.id == id {
            index = row.index - 1i64
        }
    }

    if index == -1i64 {
        let response = Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(NewScoreResponse::Error("Cannot find the newly inserted score record!".to_owned()))?;

        return Ok(response);
    }

    let response = Response::builder()
        .status(StatusCode::OK)
        .body(NewScoreResponse::Response { id: id, index: index, scores: scores })?;

    return Ok(response);
}

async fn rename_score_http(request : &Request<RenameScoreRequest>) -> anyhow::Result<Response<()>> {
    use simple_postgres::*;

    let connection = Connection::new(&load_connection_string()?);

    let body = &request.body();

    query!(
        connection,
        "UPDATE high_scores
        SET name = $1
        WHERE id = $2;",
        body.name,
        body.id)?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .body(())?;

    return Ok(response);
}

async fn authenticate(connection : &simple_postgres::Connection, request : &Request<()>) -> anyhow::Result<Response<()>> {
    use simple_postgres::*;

    fn unauthorized() -> anyhow::Result<Response<()>> {
        let response = Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .header(WWW_AUTHENTICATE, "Basic realm=\"Site Management\", charset=\"UTF-8\"")
            .body(())?;

        return Ok(response);
    }

    fn parse_credentials(credentials : &str) -> anyhow::Result<(String, String)> {
        let credentials = base64::decode(credentials)?;
        let credentials = str::from_utf8(&credentials)?;
        let mut itr = credentials.split(":");

        let login = match itr.next() {
            Some(login) => login,
            None => { return Err(anyhow!("Missing login!")); }
        };

        let password = match itr.next() {
            Some(password) => password,
            None => { return Err(anyhow!("Missing password!")); }
        };

        match itr.next() {
            Some(_) => { return Err(anyhow!("Invalid authentication header!")); }
            None => ()
        };

        return Ok((login.to_owned(), password.to_owned()));
    }

    fn get_credentials(request : &Request<()>) -> anyhow::Result<(String, String)> {
        match request.headers().get(AUTHORIZATION) {
            Some(header) => {
                let header = header.to_str()?;

                let header = match header.strip_prefix("Basic ") {
                    Some(suffix) => suffix,
                    None => { return Err(anyhow!("Unknown authorization header!")); }
                };

                return parse_credentials(header);
            }
            None => { return Err(anyhow!("Missing authorization header!")); }
        }
    }

    let (login, password) = get_credentials(request)?;

    #[derive(Debug, Deserialize)]
    struct Row {
        name : String,
        hash : String
    }

    let rows : Vec<Row> = query!(connection, "SELECT * FROM acquire_password_hash($1::varchar(128));", login)?;

    if let Some(row) = rows.iter().next() {
        let hash = &row.hash;
        let verified = argon2::verify_encoded(hash.as_str(), password.as_bytes())?;

        if !verified {
            return unauthorized();
        }

        let response = Response::builder()
            .status(StatusCode::OK)
            .body(())?;

        return Ok(response);
    }
    else {
        return Err(anyhow!("User not found!"));
    }
}

async fn initialize(request : &Request<()>) -> anyhow::Result<Response<()>> {
    use simple_postgres::*;

    let connection = Connection::new(&load_connection_string()?);

    let response = authenticate(&connection, request).await?;

    if response.status() != StatusCode::OK {
        return Ok(response);
    }

    create_default_high_scores(&connection).await?;

    let sql = include_str!("main.sql");

    query!(connection, sql)?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .body(())?;

    return Ok(response);
}

fn get_http_host() -> Result<String, anyhow::Error> {
    const HTTP_HOST : &'static str = "HTTP_HOST";

    let host : String = match env::var(HTTP_HOST) {
        Ok(value) => Ok(value),
        Err(env::VarError::NotPresent) => Ok("".to_owned()),
        Err(error @ env::VarError::NotUnicode(_)) => Err(error)
    }?;

    return Ok(host);
}

fn load_connection_string() -> Result<String, anyhow::Error> {
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

fn log_error<S: AsRef<str> + std::fmt::Display>(message : S) {
    const HTTP_HOST : &'static str = "HTTP_HOST";

    let use_stderr : bool = match env::var(HTTP_HOST) {
        Ok(value) => value.contains("localhost"),
        Err(_) => true
    };

    if use_stderr {
        eprintln!("[error] {}", message);
    }
    else {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open("error.log")
            .unwrap();

        let _ = writeln!(file, "{}", message);
        let _ = file.flush();
    }
}

fn print_output<T : serde::Serialize>(response : &anyhow::Result<Response<T>>) -> anyhow::Result<()> {

    match response {
        Ok(response) => {
            println!("Content-Type: application/json");
            println!("Status: {}\n", response.status().as_u16());
            println!("{}", serde_json::to_string(&response.body())?);
        },
        Err(error) => {
            println!("Content-Type: application/json");
            println!("Status: 400\n");
            log_error(format!("{}", error.to_string()));
        }
    }

    return Ok(());
}

fn deserialize<T>(request : Request<String>) -> anyhow::Result<Request<T>>
    where for<'de> T: de::Deserialize<'de>
{
    let (parts, body) = request.into_parts();
    let body = if body == "" { "null".to_owned() } else { body };
    let body = serde_json::from_str(&body)?;
    Ok(Request::from_parts(parts, body))
}

async fn inner_main() -> Result<(), anyhow::Error> {
    let request = get_request()?;

    // Database connection not required.
    match (request.method().as_str(), request.uri().path()) {
        ("GET", "/api/session-id/new") => print_output(&new_session_id().await)?,
        ("POST", "/api/score/new") => print_output(&new_score_http(&deserialize(request)?).await)?,
        ("POST", "/api/admin/initialize") => print_output(&initialize(&deserialize(request)?).await)?,
        ("POST", "/api/score/list") => print_output(&list_scores_http(&deserialize(request)?).await)?,
        ("POST", "/api/score/rename") => print_output(&rename_score_http(&deserialize(request)?).await)?,
        _ => ()
    };

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let result = inner_main().await;

    return match result {
        Err(error) => {
            log_error(format!("{}", error.to_string()));
            Err(error)
        },
        Ok(()) => Ok(())
    };
}
