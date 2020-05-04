use anyhow::*;
use std::process::{Command, Stdio};
use std::path::PathBuf;
use std::env;
use std::str;
use std::io::{self, Write};
use tokio_postgres::{Client, NoTls};

#[cfg(debug_assertions)]
fn debug_or_release() -> &'static str {
    return "debug";
}

#[cfg(not(debug_assertions))]
fn debug_or_release() -> &'static str {
    return "release";
}

fn issue_api_request(
    method : &str,
    uri : &str,
    content : &str) -> Result<String> {
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

    return Ok(content.to_owned());
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
        issue_api_request("POST", "/api/score/add", r#"{ "player": "Maxymilian TheBest", "score": 1000 }"#)?;
        issue_api_request("POST", "/api/score/add", r#"{ "player": "Second Player", "score": 4 }"#)?;
        issue_api_request("POST", "/api/score/add", r#"{ "player": "Third Player", "score": 3 }"#)?;
        issue_api_request("POST", "/api/score/add", r#"{ "player": "Fourth Player", "score": 2 }"#)?;
        
        let actual = issue_api_request("GET", "/api/score/list", "")?;

        let expected = r#"[
            { "player": "Maxymilian TheBest", "score": 1000 },
            { "player": "Second Player", "score": 4 },
            { "player": "Third Player", "score": 3 },
            { "player": "Fourth Player", "score": 2 }
        ]"#;

        assert_json_eq(expected, actual.as_str());

        return Ok(());
    };

    with_database("rusty_games", body).await?;

    Ok(())
}

