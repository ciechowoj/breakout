use rand_core::RngCore;
use rand_core::SeedableRng;

fn new_password(length : usize) -> String {
    let alphabet = b"abcdefghijklmnopqrstuvwxyz0123456789";

    let mut result = String::new();

    let mut x = 0;
    let mut y = 0;

    let mut osrng = rand_core::OsRng;
    let mut csprng = rand_chacha::ChaCha20Rng::from_entropy();

    while result.len() < length {
        if x < alphabet.len() as u64 {
            x = osrng.next_u64();
        }

        if y < alphabet.len() as u64 {
            y = csprng.next_u64();
        }

        result.push(alphabet[(x % alphabet.len() as u64) as usize] as char);
        x = x % alphabet.len() as u64;

        if result.len() < length {
            result.push(alphabet[(y % alphabet.len() as u64) as usize] as char);
            y = y % alphabet.len() as u64;
        }
    }

    return result;
}

#[derive(Debug)]
struct Credentials {
    database : String,
    username : String,
    password : String
}

impl Credentials {
    fn new() -> Self {
        return Credentials {
            database: String::new(),
            username: String::new(),
            password: String::new()
        };
    }
}

fn load_credentials() -> Credentials {
    let home = std::env::var("HOME").unwrap();
    let connection_string = std::fs::read_to_string(
        format!("{}{}", home, "/load_connection_string.rs"))
        .unwrap();

    let connection_string = connection_string
        .lines()
        .filter(|x| x.contains("rusty-games.eu"))
        .next()
        .unwrap()
        .trim();

    let connection_string = &connection_string
        [connection_string.find("=>").unwrap() + 2..]
        .trim_matches(|c| "\", ".contains(c));

    let properties = connection_string.split_whitespace();

    let mut result = Credentials::new();

    for property in properties {
        if let Some(value) = property.strip_prefix("dbname=") {
            result.database = value.to_owned();
        }
        else if let Some(value) = property.strip_prefix("user=") {
            result.username = value.to_owned();
        }
        else if let Some(value) = property.strip_prefix("password=") {
            result.password = value.to_owned();
        }
    }

    return result;
}

fn run_python(script : &str) -> std::process::Output {
    use rand::{thread_rng, Rng};
    use rand::distributions::Alphanumeric;

    let prefix : String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .collect();

    let path = format!("~/{}-admin-reset.py", prefix);

    let output = std::process::Command::new("ssh")
        .arg("serwer2020789")
        .arg("echo")
        .arg(base64::encode(script))
        .arg("|")
        .arg("base64")
        .arg("-d")
        .arg(">")
        .arg(&path)
        .arg("&&")
        .arg("python3")
        .arg(&path)
        .arg("&&")
        .arg("rm")
        .arg(&path)
        .output()
        .unwrap();

    return output;
}

fn main() {
    let password = new_password(20);
    let mut salt : [u8; 32] =  [0; 32];
    rand_core::OsRng.fill_bytes(&mut salt);

    let config = argon2::Config::default();
    let encoded = argon2::hash_encoded(password.as_bytes(), &salt, &config).unwrap();

    let credentials = load_credentials();

    let sql_source = include_str!("main.sql")
        .replace("{{hash}}", encoded.as_str());

    let sql_source_base64 = base64::encode(sql_source.as_bytes());

    let py_source = include_str!("main.py")
        .replace("{{admin_reset.sql}}", sql_source_base64.as_str())
        .replace("{{DBNAME}}", credentials.database.as_str())
        .replace("{{USERNAME}}", credentials.username.as_str())
        .replace("{{PASSWORD}}", credentials.password.as_str());

    let output = run_python(py_source.as_str());

    println!("Generated password: {}", password);
    print!("{}", std::str::from_utf8(&output.stdout).unwrap());
    print!("{}", std::str::from_utf8(&output.stderr).unwrap());
    println!("{}", output.status);
}
