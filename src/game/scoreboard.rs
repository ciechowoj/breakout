use web_sys::*;
use wasm_bindgen::JsCast;
use crate::utils::*;
use crate::dom_utils::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PlayerScore {
    pub player : String,
    pub score : u64
}

impl PlayerScore {
    pub fn test_scores() -> Vec<PlayerScore> {
        return vec![
            PlayerScore { player: "Maximilian".to_owned(), score: 65500 },
            PlayerScore { player: "Wedrowycz".to_owned(), score: 32700 },
            PlayerScore { player: "Super Saiyan".to_owned(), score: 16300 },
            PlayerScore { player: "Portter".to_owned(), score: 8100 },
            PlayerScore { player: "McPutin".to_owned(), score: 4000 },
            PlayerScore { player: "Hayneman".to_owned(), score: 2000 },
            PlayerScore { player: "Dolan Trump".to_owned(), score: 1000 },
            PlayerScore { player: "Mumuzaki".to_owned(), score: 500 },
            PlayerScore { player: "Erdargan".to_owned(), score: 2 },
            PlayerScore { player: "J-Ducky".to_owned(), score: 1 }
        ];
    }
}

pub fn create_scoreboard(
    document : &Document,
    overlay : &HtmlElement,
    new_score : u64) -> Expected<()> {
    let scores = load_scores()?;

    fn make_row(score : &PlayerScore, editable : bool) -> String {
        let name = if editable {
            "<input type=\"text\" id=\"score-board-input\" placeholder=\"<Your Nickname>\">".to_owned()
        }
        else {
            score.player.to_string()
        };

        return format!("<tr><td class=\"player-name\">{}</td><td class=\"player-score\">{}</td></tr>", name, score.score);
    }

    let header_str = "<tr><th colspan=\"2\">High Scores</th></tr>";

    let mut scoreboard_str = "<table>".to_string();
    scoreboard_str.push_str(header_str);

    let mut inserted = false;
    let mut num_inserted = 0;
    for score in scores {
        if !inserted && score.score < new_score {
            scoreboard_str.push_str(make_row(&PlayerScore { player: "".to_owned(), score: new_score }, true).as_str());
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

    let score_board = create_html_element(&document, "div", "score-board")?;
    score_board.set_inner_html(scoreboard_str.as_str());
    overlay.append_child(&score_board)?;

    return Ok(());
}

pub fn player_name() -> Expected<Option<String>> {
    let window = window().ok_or(Error::Msg("Failed to get window!"))?;
    let document = window.document().expect("Failed to get the main document!");
    
    let input_or_none = try_get_html_element_by_id(&document, "score-board-input")?;

    if let Some(input) = input_or_none {
        let input = input.dyn_into::<web_sys::HtmlInputElement>()
            .map_err(|_| Error::Msg("Failed to cast 'HtmlElement' to 'HtmlInputElement'."))?;

        let value = input.value();

        if !value.is_empty() {
            return Ok(Some(value));
        }
    };

    return Ok(None);
}

pub fn load_scores_from_local_storage() -> Expected<Option<Vec<PlayerScore>>> {
    let window = window().ok_or(Error::Msg("Failed to get window!"))?;
    let local_storage = window.local_storage()?.ok_or(Error::Msg("Failed to get local_storage!"))?;
    let high_scores = local_storage.get_item("high-scores")?;

    return match high_scores {
        Some(string) => {
            let result: Vec<PlayerScore> = serde_json::from_str(&string)?;
            Ok(Some(result))
        },
        None => Ok(None)
    };
}

pub fn load_scores() -> Expected<Vec<PlayerScore>> {
    let scores = load_scores_from_local_storage()?;
    let mut scores = match scores {
        Some(list) => list,
        None => PlayerScore::test_scores()
    };

    scores.sort_by(|a, b| b.score.cmp(&a.score));

    return Ok(scores);
}

pub fn save_scores_to_local_storage(high_scores : Vec<PlayerScore>) -> Expected<()> {
    let window = window().ok_or(Error::Msg("Failed to get window!"))?;
    let local_storage = window.local_storage()?.ok_or(Error::Msg("Failed to get local_storage!"))?;

    let string = serde_json::to_string(&high_scores)?;
    local_storage.set_item("high-scores", string.as_str())?;

    return Ok(());
}

pub fn persist_score(name : String, new_score : u64) -> Expected<()> {
    let mut scores = load_scores()?;

    let mut index : usize = scores.len();
    
    for i in 0..scores.len() {
        if scores[i].score < new_score {
            index = i;
            break;
        }
    }

    let player_score = PlayerScore { 
        player: name,
        score: new_score
    };
    
    scores.insert(index, player_score);
    
    save_scores_to_local_storage(scores)?;

    return Ok(());
}

