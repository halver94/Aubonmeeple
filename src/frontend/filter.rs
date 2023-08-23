use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::game::{Game, Games};

use super::server::State;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Filters {
    pub city: Option<String>,
    pub name: Option<String>,
    pub pro: bool,
}

impl Filters {
    pub fn filter(&self, games: Arc<std::sync::Mutex<Games>>) -> Vec<Box<Game>> {
        if self.city.is_none() && self.name.is_none() {
            return games.lock().unwrap().games.clone();
        }

        let filtered_games: Vec<Box<Game>> = games
            .lock()
            .unwrap()
            .games
            .clone()
            .into_iter()
            .filter(|game| {
                self.name.is_none()
                    || game
                        .okkazeo_announce
                        .name
                        .to_lowercase()
                        .contains(self.name.as_ref().unwrap())
            })
            .filter(|game| {
                self.city.is_none()
                    || (game.okkazeo_announce.city.is_some()
                        && game
                            .okkazeo_announce
                            .city
                            .as_ref()
                            .unwrap()
                            .to_lowercase()
                            .contains(self.city.as_ref().unwrap()))
            })
            .filter(|game| !(self.pro && game.okkazeo_announce.seller.is_pro))
            .collect();
        log::debug!("len of filtered games : {}", filtered_games.len());
        filtered_games
    }

    pub fn create_html(state: &State) -> String {
        let params = format!(
            "sort={}&page={}&per_page={}",
            state.sort.sort, state.pagination.page, state.pagination.per_page,
        );

        let html = format!(
            "{}{}{}",
            r#"
        <form id="filters">
        <input type="text" id="city" name="city" placeholder="Filter on city" ><br><br>
        <input type="text" id="name" name="name" placeholder="Filter on game name" ><br><br>
        <input type="checkbox" id="pro" name="pro"/><label for="pro">Exclude Pro sellers</label><br><br>
        <button type="button" onclick="submitForm()">Filter</button>
    </form>

    <script>
        function submitForm() {
            const city = document.getElementById("city").value;
            const name = document.getElementById("name").value;
            const pro = document.getElementById("pro").checked;
            
            const queryParams = [];
            
            if (city) {
                queryParams.push(`city=${encodeURIComponent(city)}`);
            }
            
            if (name) {
                queryParams.push(`name=${encodeURIComponent(name)}`);
            }
            
            queryParams.push(`pro=${encodeURIComponent(pro)}`);

            const queryString = queryParams.join("&");
            const urlWithParams = `/?"#,
            params,
            r#"&${queryString}`;
            
            window.location.href = urlWithParams;
        }
    </script>"#,
        );

        html
    }
}
