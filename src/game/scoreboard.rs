use crate::utils::*;
use web_sys::*;
use wasm_bindgen::JsCast;
use crate::dom_utils::*;
use crate::webapi::*;
use crate::executor::*;
use apilib::*;
use hex;
use rand::prelude::*;

pub fn generate_seed() -> anyhow::Result<u64> {
    let window = window().ok_or(anyhow::anyhow!("Failed to get window!"))?;

    let crypto = match window.crypto() {
        Ok(crypto) => Ok(crypto),
        Err(js_value) => Err(anyhow::anyhow!("Failed to get windows crypto {:?}!", js_value))
    }?;

    let mut seed_bytes = [0u8; 8];

    match crypto.get_random_values_with_u8_array(&mut seed_bytes) {
        Ok(_) => Ok(()),
        Err(js_value) => Err(anyhow::anyhow!("Failed to get an array of random values, inner error: {:?}!", js_value))
    }?;

    return Ok(u64::from_le_bytes(seed_bytes));
}

pub async fn proof_of_work_async(session_id : [u8; 32], seed : u64, degree : usize) -> [u8; 32] {
    let mut rng = StdRng::seed_from_u64(seed);

    loop {
        let test = rand256(&mut rng);

        let witness = async {
            validate_proof_of_work(session_id, test, degree)
        };

        yield_now().await;

        let witness = witness.await;

        if witness.0 {
            return test;
        }
    }
}

pub async fn create_scoreboard_inner(
    overlay : &HtmlElement,
    new_score : i64,
    score_board_id : String) -> anyhow::Result<()> {

    log!("Getting session id...");

    let session_id = new_session_id().await?;

    log!("session id: {:?}", session_id);

    let mut decoded_session_id = [0u8; 32];
    hex::decode_to_slice(session_id.as_str(), &mut decoded_session_id)?;

    let proof = proof_of_work_async(decoded_session_id, generate_seed()?, 8).await;
    
    log!("proof of work: {:?}", hex::encode_upper(proof));

    let response = crate::webapi::new_score(&NewScoreRequest {
        score : new_score,
        session_id : session_id,
        proof_of_work : hex::encode_upper(proof),
        limit : 10i64
    }).await?;

    log!("new score response: {:?}", response);

    match response {
        NewScoreResponse::Response { id: _, index, scores } =>
            create_scoreboard_html(overlay, index, scores, score_board_id).await?,
        NewScoreResponse::Error(_) =>
            ()
    };

    return Ok(());
}

pub async fn create_scoreboard_html(
    overlay : &HtmlElement,
    index : i64,
    scores : Vec<PlayerScore>,
    score_board_id : String) -> anyhow::Result<()> {
    let header_str = "<tr><th colspan=\"2\">High Scores</th></tr>";

    let mut scoreboard_str = "<table>".to_string();
    scoreboard_str.push_str(header_str);

    for score in scores {
        let name = if score.index == index {
            "<input type=\"text\" id=\"score-board-input\" placeholder=\"<Your Nickname>\">".to_owned()
        }
        else {
            score.name
        };

        let row = format!("<tr>\
            <td class=\"player-name\">{}. {}</td>\
            <td class=\"player-score\">{}</td>\
            </tr>",
            score.index + 1,
            name,
            score.score);

        scoreboard_str.push_str(row.as_ref());
    }

    scoreboard_str.push_str("</table>");

    let document = overlay
        .owner_document()
        .ok_or(anyhow::anyhow!("Failed to get document node."))?;

    let score_board = get_html_element_by_id(&document, score_board_id.as_ref())?;
    score_board.set_inner_html(scoreboard_str.as_str());
    overlay.append_child(&score_board).to_anyhow()?;

    return Ok(());
}

pub async fn populate_scoreboard(
    overlay : HtmlElement,
    new_score : i64,
    score_board_id : String) {
    let result1 = create_scoreboard_inner(&overlay, new_score, score_board_id.clone()).await;

    match result1 {
        Err(error) => log!("Failed to create scoreboard: {:?}", error),
        Ok(_) => ()
    }
}

pub fn create_scoreboard(
    overlay : HtmlElement,
    new_score : i64,
    score_board_id : &str) -> anyhow::Result<()> {
    let document = overlay
        .owner_document()
        .ok_or(anyhow::anyhow!("Failed to get document node."))?;

    let score_board = create_html_element(&document, "div", score_board_id)?;
    overlay.append_child(&score_board).to_anyhow()?;

    let future = populate_scoreboard(overlay.clone(), new_score, score_board_id.to_owned());

    execute(future);

    return Ok(());
}

pub fn player_name() -> anyhow::Result<Option<String>> {
    let window = window().ok_or(anyhow::anyhow!("Failed to get window!"))?;
    let document = window.document().expect("Failed to get the main document!");
    
    let input_or_none = try_get_html_element_by_id(&document, "score-board-input")?;

    if let Some(input) = input_or_none {
        let input = input.dyn_into::<web_sys::HtmlInputElement>()
            .map_err(|_| anyhow::anyhow!("Failed to cast 'HtmlElement' to 'HtmlInputElement'."))?;

        let value = input.value();

        if !value.is_empty() {
            return Ok(Some(value));
        }
    };

    return Ok(None);
}

pub fn _load_scores_from_local_storage() -> anyhow::Result<Option<Vec<PlayerScore>>> {
    let window = window().ok_or(anyhow::anyhow!("Failed to get window!"))?;

    let local_storage = window
        .local_storage()
        .to_anyhow()?
        .ok_or(anyhow::anyhow!("Failed to get local_storage!"))?;

    let high_scores = local_storage.get_item("high-scores").to_anyhow()?;

    return match high_scores {
        Some(string) => {
            let result: Vec<PlayerScore> = serde_json::from_str(&string)?;
            Ok(Some(result))
        },
        None => Ok(None)
    };
}

pub async fn load_scores() -> anyhow::Result<Vec<PlayerScore>> {
    let scores = list_scores_http(&ListScoresRequest { limit: Some(10) }).await?;
    
    if scores.status() != http::status::StatusCode::OK {
        return Err(anyhow::anyhow!("Failed to list scores."));
    }

    let mut scores = scores.into_body();

    scores.sort_by(|a, b| b.score.cmp(&a.score));

    return Ok(scores);
}

pub fn save_scores_to_local_storage(high_scores : Vec<PlayerScore>) -> anyhow::Result<()> {
    let window = window()
        .ok_or(anyhow::anyhow!("Failed to get window!"))?;

    let local_storage = window
        .local_storage()
        .to_anyhow()?
        .ok_or(anyhow::anyhow!("Failed to get local_storage!"))?;

    let string = serde_json::to_string(&high_scores)?;
    local_storage.set_item("high-scores", string.as_str()).to_anyhow()?;

    return Ok(());
}

pub async fn persist_score_inner(name : String, new_score : i64) -> anyhow::Result<()> {
    let mut scores = load_scores().await?;

    let mut index : usize = scores.len();
    
    for i in 0..scores.len() {
        if scores[i].score < new_score {
            index = i;
            break;
        }
    }

    let player_score = PlayerScore { 
        index: index as i64,
        name: name,
        score: new_score
    };
    
    scores.insert(index, player_score);

    save_scores_to_local_storage(scores)?;

    return Ok(());
}

pub async fn persist_score(name : String, new_score : i64) {
    let _result = persist_score_inner(name, new_score).await;
}
