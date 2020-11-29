use anyhow::*;
use std::process::{Command, Stdio};
use std::path::PathBuf;
use std::env;
use std::str;
use std::io::{self, Write};
use tokio_postgres::{Client, NoTls};
use tokio::sync::Semaphore;
use http::{Response, StatusCode};
use lazy_static::lazy_static;
use apilib::*;

lazy_static! {
    static ref TEST_SEMAPHORE : Semaphore = Semaphore::new(1);
}

#[cfg(debug_assertions)]
fn debug_or_release() -> &'static str {
    return "debug";
}

#[cfg(not(debug_assertions))]
fn debug_or_release() -> &'static str {
    return "release";
}

fn cgi_get_api_exe() -> Result<String> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
    let pkg_name = env::var("CARGO_PKG_NAME")?;

    let mut api_exe = PathBuf::new();
    api_exe.push(manifest_dir);
    api_exe.push("target");
    api_exe.push(debug_or_release());
    api_exe.push(pkg_name);

    let api_exe = api_exe
        .as_path()
        .to_str()
        .ok_or(anyhow!("Result path contains non-unicode characters."))?;

    return Ok(api_exe.to_owned());
}

fn cgi_output_to_response<T: serde::de::DeserializeOwned>(output : std::process::Output) -> Result<Response<Option<T>>> {
    if !output.status.success() {
        println!("status: {}", output.status);
        io::stderr().write_all(&output.stderr)?;
        bail!("The cgi script finished with nonzero status!");
    }

    let stdout = str::from_utf8(&output.stdout)?;

    let mut split_itr = stdout.splitn(2, "\n\n");

    let headers = split_itr.next().unwrap();
    let content = if let Some(content) = split_itr.next() { content } else { "" };

    let mut header_itr = headers.split("\n");

    let mut content_type_present = false;
    let mut status : Option<String> = None;

    while let Some(header) = header_itr.next() {
        if header.starts_with("Content-Type: application/json") {
            content_type_present = true;
        }
        else if header.starts_with("Status") || header.starts_with("status:") {
            status = header.splitn(2, ":").last().map(|x| x.trim().to_owned());
        }
    }

    if !content_type_present {
        bail!("Content-Type header missing or is not set to 'application/json'!");
    }

    let status = match status { Some(status) => StatusCode::from_u16(status.parse()?)?, None => StatusCode::OK };

    if status == http::StatusCode::OK {
        let response : Response<Option<T>> = Response::builder()
            .status(status)
            .body(Some(serde_json::from_str(content)?))
            .unwrap();

        return Ok(response);
    }
    else {
        let response : Response<Option<T>> = Response::builder()
            .status(status)
            .body(None)
            .unwrap();

        return Ok(response);
    }
}

fn issue_api_request<T: serde::de::DeserializeOwned>(
    database : &str,
    method : &str,
    uri : &str,
    content : &str) -> Result<Response<Option<T>>> {
    let api_exe = cgi_get_api_exe()?;
    let api_exe = api_exe.as_str();

    let mut split_itr = uri.splitn(2, '?');
    split_itr.next();

    let query_string = match split_itr.next() {
        Some(x) => x,
        None => ""
    };

    let mut process = Command::new(api_exe)
        .env("DATABASE_NAME", database)
        .env("SESSION_ID_SALT", "0000000000000000000000000000000000000000000000000000000000000000")
        .env("CONTENT_LENGTH", content.len().to_string())
        .env("REQUEST_METHOD", method)
        .env("REQUEST_URI", uri)
        .env("QUERY_STRING", query_string)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    process.stdin
        .as_mut()
        .unwrap()
        .write_all(content.as_bytes())?;

    let output = process.wait_with_output()?;

    return cgi_output_to_response(output);
}

fn issue_api_request_with_authz<T: serde::de::DeserializeOwned>(
    database : &str,
    method : &str,
    uri : &str,
    content : &str,
    credentials : &str) -> Result<Response<Option<T>>> {
    let api_exe = cgi_get_api_exe()?;
    let api_exe = api_exe.as_str();

    let mut split_itr = uri.splitn(2, '?');
    split_itr.next();

    let query_string = match split_itr.next() {
        Some(x) => x,
        None => ""
    };

    let credentials = format!("Basic {}", base64::encode(credentials.as_bytes()));

    let mut process = Command::new(api_exe)
        .env("DATABASE_NAME", database)
        .env("SESSION_ID_SALT", "0000000000000000000000000000000000000000000000000000000000000000")
        .env("CONTENT_LENGTH", content.len().to_string())
        .env("REQUEST_METHOD", method)
        .env("REQUEST_URI", uri)
        .env("QUERY_STRING", query_string)
        .env("HTTP_AUTHORIZATION", credentials)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    process.stdin
        .as_mut()
        .unwrap()
        .write_all(content.as_bytes())?;

    let output = process.wait_with_output()?;

    return cgi_output_to_response(output);
}

async fn with_database(
    database: &'static str,
    setup_sql: Option<&'static str>,
    callback: &mut dyn FnMut(&Client) -> Result<(), Box<dyn std::error::Error>>)
    -> Result<(), Box<dyn std::error::Error>> {

    let _guard = TEST_SEMAPHORE.acquire().await;

    let connection_string = "host=localhost user=testuser dbname=testdb password=password";

    let (client, connection) =
        tokio_postgres::connect(connection_string.as_ref(), NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let row = client
        .query_opt(format!("SELECT datname FROM pg_database WHERE datname = '{}';", database).as_str(), &[])
        .await?;

    if let Some(_) = row {
        client
            .execute(format!("DROP DATABASE {};", database).as_str(), &[])
            .await?;
    }

    client
        .execute(format!("CREATE DATABASE {};", database).as_str(), &[])
        .await?;

    let connection_string = format!("host=localhost user=testuser dbname={} password=password", database);

    let (client, connection) =
        tokio_postgres::connect(connection_string.as_ref(), NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let sql = include_str!("../src/main.sql");

    client
        .batch_execute(sql)
        .await?;

    if let Some(sql) = setup_sql {
        client
            .batch_execute(sql)
            .await?;
    }

    with_root_password(&client).await?;

    callback(&client)?;

    /* client
        .execute(format!("DROP DATABASE {};", database).as_str(), &[])
        .await?;*/

    return Ok(());
}

async fn with_root_password(client : &Client) -> anyhow::Result<()> {
    let config = argon2::Config::default();
    let encoded = argon2::hash_encoded("password".as_bytes(), "saltsaltsalt".as_bytes(), &config).unwrap();

    client
        .execute("SELECT upsert_password($1, $2);", &[&"root", &encoded])
        .await?;

    return Ok(());
}

fn assert_json_eq(a : &str, b : &str) {
    let a : String = a.chars().filter(|c| !c.is_whitespace()).collect();
    let b : String = b.chars().filter(|c| !c.is_whitespace()).collect();
    assert_eq!(a.as_str(), b.as_str());
}

#[tokio::test]
async fn new_session_id_test() -> Result<(), Box<dyn std::error::Error>> {
    let actual : Response<Option<String>> = issue_api_request("new_session_id_test", "GET", "/api/session-id/new", r#""#)?;

    assert_eq!(StatusCode::OK, actual.status());
    assert_ne!(None, *actual.body());
    assert_ne!(Some("".to_owned()), *actual.body());

    Ok(())
}

#[tokio::test]
async fn test_init_checks_admin_password() -> Result<(), Box<dyn std::error::Error>> {
    let body : &mut dyn FnMut(&Client) -> Result<(), Box<dyn std::error::Error>> = &mut |_| {
        let response : Response<Option<()>> = issue_api_request(
            "test_init_checks_admin_password",
            "POST",
            "/api/admin/initialize",
            r#""#)?;

        assert_eq!(StatusCode::BAD_REQUEST, response.status());

        let response : Response<Option<()>> = issue_api_request_with_authz(
            "test_init_checks_admin_password",
            "POST",
            "/api/admin/initialize",
            r#""#,
            "root:password")?;

        assert_eq!(StatusCode::OK, response.status());

        let response : Response<Option<()>> = issue_api_request_with_authz(
            "test_init_checks_admin_password",
            "POST",
            "/api/admin/initialize",
            r#""#,
            "root:wrong-password")?;

        assert_eq!(StatusCode::UNAUTHORIZED, response.status());

        return Ok(());
    };

    with_database("test_init_checks_admin_password", None, body).await?;

    Ok(())
}

#[tokio::test]
async fn test_new_rename_api() -> Result<(), Box<dyn std::error::Error>> {
    let body : &mut dyn FnMut(&Client) -> Result<(), Box<dyn std::error::Error>> = &mut |_| {
        let session_id : Response<Option<String>> = issue_api_request("new_session_id_test", "GET", "/api/session-id/new", r#""#)?;

        assert_eq!(StatusCode::OK, session_id.status());

        let mut decoded_session_id = [0u8; 32];
        hex::decode_to_slice(session_id.body().as_ref().unwrap(), &mut decoded_session_id)?;
        let proof_of_work = proof_of_work(decoded_session_id, 42u64, 8);

        let request = NewScoreRequest {
            score: 85i64,
            session_id: session_id.body().as_ref().unwrap().clone(),
            proof_of_work: hex::encode_upper(proof_of_work),
            limit: 4i64
        };

        let request_json = serde_json::to_string(&request)?;

        let actual : Response<Option<NewScoreResponse>> = issue_api_request(
            "test_new_rename_api",
            "POST",
            "/api/score/new",
            request_json.as_str())?;

        let expected = r#"[
            { "index": 0, "name": "First Player", "score": 100 },
            { "index": 1, "name": "Second Player", "score": 90 },
            { "index": 2, "name": "", "score": 85 },
            { "index": 3, "name": "Third Player", "score": 80 }
        ]"#;

        assert_eq!(StatusCode::OK, actual.status());

        let id = match actual.body().as_ref().unwrap() {
            NewScoreResponse::Response { id, index: _, scores } => {
                assert_json_eq(expected, serde_json::to_string(&scores)?.as_str());
                Ok(id)
            },
            NewScoreResponse::Error(error) => Err(anyhow!("{}", error))
        }?;

        let actual : Response<Option<()>> = issue_api_request(
            "test_new_rename_api",
            "POST",
            "/api/score/rename",
            format!(r#"{{ "id": "{}", "name": "New Player" }}"#, id).as_str())?;

        assert_eq!(StatusCode::OK, actual.status());

        let actual : Response<Option<Vec<PlayerScore>>> = issue_api_request(
            "test_new_rename_api",
            "POST",
            "/api/score/list",
            r#"{ "limit": 4 }"#)?;

        assert_eq!(StatusCode::OK, actual.status());

        let expected = r#"[
            { "index": 0, "name": "First Player", "score": 100 },
            { "index": 1, "name": "Second Player", "score": 90 },
            { "index": 2, "name": "New Player", "score": 85 },
            { "index": 3, "name": "Third Player", "score": 80 }
        ]"#;

        assert_eq!(StatusCode::OK, actual.status());
        assert_json_eq(expected, serde_json::to_string(&actual.body())?.as_str());

        return Ok(());
    };

    with_database("test_new_rename_api", Some("SELECT * FROM insert_dummy_scores();"), body).await?;

    Ok(())
}

#[tokio::test]
async fn test_new_rename_api_return_records_from_the_middle() -> Result<(), Box<dyn std::error::Error>> {
    let body : &mut dyn FnMut(&Client) -> Result<(), Box<dyn std::error::Error>> = &mut |_| {
        let session_id : Response<Option<String>> = issue_api_request("test_new_rename_api_return_records_from_the_middle", "GET", "/api/session-id/new", r#""#)?;

        assert_eq!(StatusCode::OK, session_id.status());

        let mut decoded_session_id = [0u8; 32];
        hex::decode_to_slice(session_id.body().as_ref().unwrap(), &mut decoded_session_id)?;
        let proof_of_work = proof_of_work(decoded_session_id, 42u64, 8);

        let request = NewScoreRequest {
            score: 55i64,
            session_id: session_id.body().as_ref().unwrap().clone(),
            proof_of_work: hex::encode_upper(proof_of_work),
            limit: 4i64
        };

        let request_json = serde_json::to_string(&request)?;

        let actual : Response<Option<NewScoreResponse>> = issue_api_request(
            "test_new_rename_api_return_records_from_the_middle",
            "POST",
            "/api/score/new",
            request_json.as_str())?;

        let expected = r#"[
            { "index": 3, "name": "Fourth Player", "score": 70 },
            { "index": 4, "name": "Fifth Player", "score": 60 },
            { "index": 5, "name": "", "score": 55 },
            { "index": 6, "name": "Sixth Player", "score": 50 }
        ]"#;

        assert_eq!(StatusCode::OK, actual.status());

        match actual.body().as_ref().unwrap() {
            NewScoreResponse::Response { id, index: _, scores } => {
                assert_json_eq(expected, serde_json::to_string(&scores)?.as_str());
                Ok(id)
            },
            NewScoreResponse::Error(error) => Err(anyhow!("{}", error))
        }?;

        return Ok(());
    };

    with_database("test_new_rename_api_return_records_from_the_middle", Some("SELECT * FROM insert_dummy_scores();"), body).await?;

    Ok(())
}

#[tokio::test]
async fn test_new_rename_api_invalid_session_id() -> Result<(), Box<dyn std::error::Error>> {
    let body : &mut dyn FnMut(&Client) -> Result<(), Box<dyn std::error::Error>> = &mut |_| {
        let session_id : Response<Option<String>> = issue_api_request("test_new_rename_api_invalid_session_id", "GET", "/api/session-id/new", r#""#)?;

        assert_eq!(StatusCode::OK, session_id.status());

        let session_id_amended = session_id.body().as_ref().unwrap()
            .char_indices()
            .map(|(i, c)| if i == 0 { if c == '0' { '1' } else { '0' } } else { c })
            .collect::<String>();

        let request = NewScoreRequest {
            score: 85i64,
            session_id: session_id_amended,
            proof_of_work: "".to_owned(),
            limit: 4i64
        };

        let request_json = serde_json::to_string(&request)?;

        let actual : Response<Option<NewScoreResponse>> = issue_api_request(
            "test_new_rename_api_invalid_session_id",
            "POST",
            "/api/score/new",
            request_json.as_str())?;

        assert_eq!(StatusCode::BAD_REQUEST, actual.status());

        return Ok(());
    };

    with_database("test_new_rename_api_invalid_session_id", Some("SELECT * FROM insert_dummy_scores();"), body).await?;

    Ok(())
}

#[tokio::test]
async fn test_new_rename_api_session_id_cannot_be_reused() -> Result<(), Box<dyn std::error::Error>> {
    let body : &mut dyn FnMut(&Client) -> Result<(), Box<dyn std::error::Error>> = &mut |_| {
        let session_id : Response<Option<String>> = issue_api_request("test_new_rename_api_session_id_cannot_be_reused", "GET", "/api/session-id/new", r#""#)?;

        assert_eq!(StatusCode::OK, session_id.status());

        let mut decoded_session_id = [0u8; 32];
        hex::decode_to_slice(session_id.body().as_ref().unwrap(), &mut decoded_session_id)?;
        let proof_of_work = proof_of_work(decoded_session_id, 42u64, 8);

        let request = NewScoreRequest {
            score: 85i64,
            session_id: session_id.body().as_ref().unwrap().clone(),
            proof_of_work: hex::encode_upper(proof_of_work),
            limit: 4i64
        };

        let request_json = serde_json::to_string(&request)?;

        let actual : Response<Option<NewScoreResponse>> = issue_api_request(
            "test_new_rename_api_session_id_cannot_be_reused",
            "POST",
            "/api/score/new",
            request_json.as_str())?;

        assert_eq!(StatusCode::OK, actual.status());

        let actual : Response<Option<NewScoreResponse>> = issue_api_request(
            "test_new_rename_api_session_id_cannot_be_reused",
            "POST",
            "/api/score/new",
            request_json.as_str())?;

        assert_eq!(StatusCode::BAD_REQUEST, actual.status());

        return Ok(());
    };

    with_database("test_new_rename_api_session_id_cannot_be_reused", Some("SELECT * FROM insert_dummy_scores();"), body).await?;

    Ok(())
}

#[tokio::test]
async fn test_new_rename_api_invalid_proof_of_work() -> Result<(), Box<dyn std::error::Error>> {
    let body : &mut dyn FnMut(&Client) -> Result<(), Box<dyn std::error::Error>> = &mut |_| {
        let session_id : Response<Option<String>> = issue_api_request("test_new_rename_api_invalid_proof_of_work", "GET", "/api/session-id/new", r#""#)?;

        assert_eq!(StatusCode::OK, session_id.status());

        let request = NewScoreRequest {
            score: 85i64,
            session_id: session_id.body().as_ref().unwrap().clone(),
            proof_of_work: session_id.body().as_ref().unwrap().clone(),
            limit: 4i64
        };

        let request_json = serde_json::to_string(&request)?;

        let actual : Response<Option<NewScoreResponse>> = issue_api_request(
            "test_new_rename_api_invalid_proof_of_work",
            "POST",
            "/api/score/new",
            request_json.as_str())?;

        assert_eq!(StatusCode::BAD_REQUEST, actual.status());

        return Ok(());
    };

    with_database("test_new_rename_api_invalid_proof_of_work", Some("SELECT * FROM insert_dummy_scores();"), body).await?;

    Ok(())
}

