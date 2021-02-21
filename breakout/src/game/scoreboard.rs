use crate::utils::*;
use web_sys::*;
use wasm_bindgen::JsCast;
use crate::webapi::*;
use crate::executor::*;
use apilib::*;
use hex;
use rand::prelude::*;
use std::{rc::Rc, cell::RefCell};
use uuid::Uuid;

pub fn generate_seed() -> anyhow::Result<u64> {
    let window = web_sys::window().unwrap();

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
    score_id : Rc<RefCell<Uuid>>,
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
        NewScoreResponse::Response { id, index, scores } => {
            create_scoreboard_html(overlay, index, scores, score_board_id).await?;
            *score_id.borrow_mut() = id;
        },
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
    let header_str = "<tr><th colspan=\"2\" class=\"font-large\">High Scores</th></tr>";

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
            <td class=\"player-name font-large\">{}. {}</td>\
            <td class=\"player-score font-large\">{}</td>\
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

    let score_board = document.get_element_by_id(score_board_id.as_ref()).unwrap();
    score_board.set_inner_html(scoreboard_str.as_str());
    overlay.append_child(&score_board).to_anyhow()?;

    return Ok(());
}

pub fn collapse_scoreboard_input_html(overlay : &HtmlElement) -> anyhow::Result<()> {
    let document = overlay
        .owner_document()
        .ok_or(anyhow::anyhow!("Failed to get document node."))?;

    let scoreboard_input = document.get_element_by_id("score-board-input");
    let parent = scoreboard_input.map(|input| input.parent_element().unwrap());

    match parent {
        Some(parent) => {
            let parent : HtmlElement = parent.unchecked_into();

            let inner = parent.inner_html();

            let index = inner.split('.').next().unwrap();
            let name = player_name()?.unwrap();

            parent.set_inner_html(format!("{}. {}", index, name).as_str());
        },
        None => {}
    }

    return Ok(());
}

pub async fn populate_scoreboard(
    overlay : HtmlElement,
    new_score : i64,
    score_id : Rc<RefCell<Uuid>>,
    score_board_id : String) {
    let result1 = create_scoreboard_inner(&overlay, new_score, score_id, score_board_id.clone()).await;

    match result1 {
        Err(error) => log!("Failed to create scoreboard: {:?}", error),
        Ok(_) => ()
    }
}

pub fn create_scoreboard(
    overlay : HtmlElement,
    new_score : i64,
    score_id : Rc<RefCell<Uuid>>,
    score_board_id : &str) -> anyhow::Result<()> {
    let document = overlay
        .owner_document()
        .ok_or(anyhow::anyhow!("Failed to get document node."))?;

    let score_board : HtmlElement = document.create_element("div").unwrap().unchecked_into();
    score_board.set_id(score_board_id);

    overlay.append_child(&score_board).to_anyhow()?;

    let future = populate_scoreboard(overlay.clone(), new_score, score_id, score_board_id.to_owned());

    execute(future);

    return Ok(());
}

pub fn player_name() -> anyhow::Result<Option<String>> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let input_or_none = document.get_element_by_id("score-board-input");

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

pub async fn persist_score_inner(name : String, score_id : Uuid) -> anyhow::Result<()> {
    crate::webapi::rename_score(&RenameScoreRequest {
        id: score_id,
        name: name
    }).await?;

    return Ok(());
}

pub async fn persist_score_async(overlay : HtmlElement, name : String, score_id : Uuid) {
    let _result = persist_score_inner(name, score_id).await;
    let _result = collapse_scoreboard_input_html(&overlay);
}

pub fn persist_score(overlay : HtmlElement, name : String, score_id : Uuid) -> anyhow::Result<()> {
    let future = persist_score_async(overlay, name, score_id);
    execute(future);
    return Ok(());
}
