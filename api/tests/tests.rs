use anyhow::*;
use std::process::{Command, Stdio};
use std::path::PathBuf;
use std::env;
use std::str;
use std::io::{self, Write};
use tokio_postgres::{Client, NoTls};
use http::{Response, StatusCode};
use apilib::*;

#[cfg(debug_assertions)]
fn debug_or_release() -> &'static str {
    return "debug";
}

#[cfg(not(debug_assertions))]
fn debug_or_release() -> &'static str {
    return "release";
}

fn issue_api_request<T: serde::de::DeserializeOwned>(
    database : &str,
    method : &str,
    uri : &str,
    content : &str) -> Result<Response<T>> {
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

    let mut split_itr = uri.splitn(2, '?');
    split_itr.next();

    let query_string = match split_itr.next() {
        Some(x) => x,
        None => ""
    };

    let mut process = Command::new(api_exe)
        .env("DATABASE_NAME", database)
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

    if !output.status.success() {
        println!("status: {}", output.status);
        io::stderr().write_all(&output.stderr)?;
        bail!("The cgi script finished with nonzero status!");
    }

    let stdout = str::from_utf8(&output.stdout)?;
    let mut split_itr = stdout.splitn(3, "\n");

    let content = if let Some("Content-Type: application/json") = split_itr.next() {
        if let Some("") = split_itr.next() {
            if let Some(content) = split_itr.next() {
                content
            }
            else {
                ""
            }
        }
        else {
            bail!("Empty line expected!");    
        }
    }
    else {
        bail!("Content-Type header missing or is not set to 'application/json'!");
    };

    let response : Response<T> = Response::builder()
        .status(200)
        .body(serde_json::from_str(content)?)
        .unwrap();

    return Ok(response);
}

async fn with_database(
    database: &'static str,
    callback: &mut dyn FnMut(&Client) -> Result<(), Box<dyn std::error::Error>>) 
    -> Result<(), Box<dyn std::error::Error>> {
    let connection_string = "host=localhost user=wojciech dbname=wojciech password=password";

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

    callback(&client)?;

    /* client
        .execute(format!("DROP DATABASE {};", database).as_str(), &[])
        .await?;*/

    return Ok(());
}

fn fill_with_test_data(database : &str) -> anyhow::Result<()> {
    issue_api_request::<()>(database, "POST", "/api/score/add", r#"{ "name": "First Player", "score": 100 }"#)?;
    issue_api_request::<()>(database, "POST", "/api/score/add", r#"{ "name": "Second Player", "score": 90 }"#)?;
    issue_api_request::<()>(database, "POST", "/api/score/add", r#"{ "name": "Third Player", "score": 80 }"#)?;
    issue_api_request::<()>(database, "POST", "/api/score/add", r#"{ "name": "Fourth Player", "score": 70 }"#)?;
    issue_api_request::<()>(database, "POST", "/api/score/add", r#"{ "name": "Fifth Player", "score": 60 }"#)?;
    issue_api_request::<()>(database, "POST", "/api/score/add", r#"{ "name": "Sixth Player", "score": 50 }"#)?;
    issue_api_request::<()>(database, "POST", "/api/score/add", r#"{ "name": "Seventh Player", "score": 40 }"#)?;
    issue_api_request::<()>(database, "POST", "/api/score/add", r#"{ "name": "Eights Player", "score": 30 }"#)?;
    issue_api_request::<()>(database, "POST", "/api/score/add", r#"{ "name": "Ninth Player", "score": 20 }"#)?;
    issue_api_request::<()>(database, "POST", "/api/score/add", r#"{ "name": "Tenth Player", "score": 10 }"#)?;
    return Ok(());
}

fn assert_json_eq(a : &str, b : &str) {
    let a : String = a.chars().filter(|c| !c.is_whitespace()).collect();
    let b : String = b.chars().filter(|c| !c.is_whitespace()).collect();
    assert_eq!(a.as_str(), b.as_str());
}

// GET score/list -> [ { "player": "Maxymilian TheBest", "score": 1000 }, {}, ... ]
// POST score/add { "player": "Maxymilian TheBest", "score": 1000 }
#[tokio::test]
async fn simple_test() -> Result<(), Box<dyn std::error::Error>> {
    let body : &mut dyn FnMut(&Client) -> Result<(), Box<dyn std::error::Error>> = &mut |_| {
        let _ : Response<()> = issue_api_request("simple_test", "POST", "/api/score/add", r#"{ "name": "Maxymilian TheBest", "score": 1000 }"#)?;
        let _ : Response<()> = issue_api_request("simple_test", "POST", "/api/score/add", r#"{ "name": "Second Player", "score": 4 }"#)?;
        let _ : Response<()> = issue_api_request("simple_test", "POST", "/api/score/add", r#"{ "name": "Third Player", "score": 3 }"#)?;
        let _ : Response<()> = issue_api_request("simple_test", "POST", "/api/score/add", r#"{ "name": "Fourth Player", "score": 2 }"#)?;
        
        let actual : Response<Vec<PlayerScore>> = issue_api_request("simple_test", "GET", "/api/score/list", "")?;

        let expected = r#"[
            { "index": 0, "name": "Maxymilian TheBest", "score": 1000 },
            { "index": 1, "name": "Second Player", "score": 4 },
            { "index": 2, "name": "Third Player", "score": 3 },
            { "index": 3, "name": "Fourth Player", "score": 2 }
        ]"#;

        assert_eq!(StatusCode::OK, actual.status());
        assert_json_eq(expected, serde_json::to_string(&actual.body())?.as_str());

        return Ok(());
    };

    with_database("simple_test", body).await?;

    Ok(())
}

// POST /api/score/new (score : i64) -> uuid top 9 + id
// POST /api/score/rename (id : uuid, player : String) -> ()
#[tokio::test]
async fn test_new_rename_api() -> Result<(), Box<dyn std::error::Error>> {
    let body : &mut dyn FnMut(&Client) -> Result<(), Box<dyn std::error::Error>> = &mut |_| {
        fill_with_test_data("test_new_rename_api")?;
        
        let actual : Response<NewScoreResponse> = issue_api_request(
            "test_new_rename_api",
            "POST",
            "/api/score/new",
            r#"{ "score": 85, "limit": 4 }"#)?;
        
        let expected = r#"[
            { "index": 0, "name": "First Player", "score": 100 },
            { "index": 1, "name": "Second Player", "score": 90 },
            { "index": 2, "name": "", "score": 85 },
            { "index": 3, "name": "Third Player", "score": 80 }
        ]"#;

        assert_eq!(StatusCode::OK, actual.status());
        assert_json_eq(expected, serde_json::to_string(&actual.body().scores)?.as_str());

        let id = actual.body().id;

        let actual : Response<()> = issue_api_request(
            "test_new_rename_api",
            "POST",
            "/api/score/rename",
            format!(r#"{{ "id": {}, "name": "New Player" }}"#, id).as_str())?;

        assert_eq!(StatusCode::OK, actual.status());

        let actual : Response<Vec<PlayerScore>> = issue_api_request(
            "test_new_rename_api",
            "GET",
            "/api/score/list",
            r#"{ "limit": 4 }"#)?;

        assert_eq!(StatusCode::OK, actual.status());

        let expected = r#"[
            { "index": 0, "name": "First Player", "score": 100 },
            { "index": 1, "name": "Second Player", "score": 90 },
            { "index": 2, "name": "", "score": 85 },
            { "index": 3, "name": "Third Player", "score": 80 }
        ]"#;

        assert_eq!(StatusCode::OK, actual.status());
        assert_json_eq(expected, serde_json::to_string(&actual.body())?.as_str());

        return Ok(());
    };

    with_database("test_new_rename_api", body).await?;

    Ok(())
}
