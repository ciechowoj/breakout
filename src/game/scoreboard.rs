use web_sys::*;
use crate::utils::*;
use crate::dom_utils::*;

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
    let scores = PlayerScore::test_scores();

    fn make_row(score : &PlayerScore, editable : bool) -> String {
        let name = if editable {
            "<input type=\"text\" id=\"fname\" name=\"fname\" placeholder=\"<Your Nickname>\">".to_owned()
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
        }
        else {
            scoreboard_str.push_str(make_row(&score, false).as_str());
        }

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
