use std::env;
use std::error;
use std::io;
use std::io::Read;
use std::str::FromStr;
use tokio_postgres::{NoTls, Error};
use serde::{Serialize, Deserialize};

// GET score/list -> [ { "player": "Maxymilian TheBest", "score": 1000 }, {}, ... ]
// POST score { "player": "Maxymilian TheBest", "score": 1000 }

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    // let (client, connection) =
    //     tokio_postgres::connect("host=localhost user=wojciech dbname=wojciech password=password", NoTls).await?;

    // tokio::spawn(async move {
    //     if let Err(e) = connection.await {
    //         eprintln!("connection error: {}", e);
    //     }
    // });

    // // Now we can execute a simple statement that just returns its parameter.
    // let rows = client
    //     .query("SELECT $1::TEXT", &[&"hello world"])
    //     .await?;

    // // And then check that we got back the same string we sent over.
    // let value: &str = rows[0].get(0);
    // assert_eq!(value, "hello world");

    let serialized = serde_json::to_string(&get_request()?)?;

    println!("Content-type: application/json\n");
    println!("{}", serialized);

    Ok(())
}



