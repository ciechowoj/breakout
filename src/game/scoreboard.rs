use crate::utils::*;
use web_sys::*;
use wasm_bindgen::JsCast;
use crate::dom_utils::*;
use crate::webapi::*;
use apilib::*;

pub async fn create_scoreboard_inner(
    overlay : &HtmlElement,
    new_score : i64) -> anyhow::Result<()> {
    
    fn make_row(score : &PlayerScore, editable : bool) -> String {
        let name = if editable {
            "<input type=\"text\" id=\"score-board-input\" placeholder=\"<Your Nickname>\">".to_owned()
        }
        else {
            score.name.to_string()
        };

        return format!("<tr><td class=\"player-name\">{}</td><td class=\"player-score\">{}</td></tr>", name, score.score);
    }

    let header_str = "<tr><th colspan=\"2\">High Scores</th></tr>";

    let mut scoreboard_str = "<table>".to_string();
    scoreboard_str.push_str(header_str);

    let scores = load_scores().await?;

    let mut inserted = false;
    let mut num_inserted = 0;
    for score in scores {
        if !inserted && score.score < new_score {
            scoreboard_str.push_str(make_row(&PlayerScore { index: num_inserted, name: "".to_owned(), score: new_score }, true).as_str());
            inserted = true;

            num_inserted += 1;
            if num_inserted == 10 {
                break;
            }
        }
        
        scoreboard_str.push_str(make_row(&score, false).as_str());
        
        num_inserted += 1;
        if num_inserted == 10 {
            break;
        }
    }

    scoreboard_str.push_str("</table>");

    let document = overlay
        .owner_document()
        .ok_or(anyhow::anyhow!("Failed to get document node."))?;

    let score_board = create_html_element(&document, "div", "score-board")?;
    score_board.set_inner_html(scoreboard_str.as_str());
    overlay.append_child(&score_board).to_anyhow()?;

    return Ok(());
}

pub async fn create_scoreboard(
    overlay : HtmlElement,
    new_score : i64) {
    let _result = create_scoreboard_inner(&overlay, new_score).await;
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
