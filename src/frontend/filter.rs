use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::game::{Game, Games};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filters {
    city: Option<String>,
    name: Option<String>,
}

impl Default for Filters {
    fn default() -> Self {
        Self {
            city: None,
            name: None,
        }
    }
}

impl Filters {
    pub fn filter(self, games: Arc<std::sync::Mutex<Games>>) -> Vec<Game> {
        println!("filters : {:#?}", self);

        if self.city.is_none() && self.name.is_none() {
            return games.lock().unwrap().games.clone();
        }

        let filtered_games: Vec<Game> = games
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
            .collect();
        println!("len of filtered games : {}", filtered_games.len());
        filtered_games
    }

    pub fn create_html() -> String {
        let html = r#"
        <form id="filters">
        <input type="text" id="city" name="city" placeholder="Filter on city" ><br><br>
        <input type="text" id="name" name="name" placeholder="Filter on game name" ><br><br>
        <button type="button" onclick="submitForm()">Filter</button>
    </form>

    <script>
        function submitForm() {
            const city = document.getElementById("city").value;
            const name = document.getElementById("name").value;
            
            const queryParams = [];
            
            if (city) {
                queryParams.push(`city=${encodeURIComponent(city)}`);
            }
            
            if (name) {
                queryParams.push(`name=${encodeURIComponent(name)}`);
            }
            
            const queryString = queryParams.join("&");
            const urlWithParams = `/?${queryString}`;
            
            window.location.href = urlWithParams;
        }
    </script>"#;

        html.to_string()
    }
}

/*<!DOCTYPE html>
<html>
<head>
<link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/4.7.0/css/font-awesome.min.css">
<style>
* {box-sizing: border-box;}

body {
  margin: 0;
  font-family: Arial, Helvetica, sans-serif;
}

.topnav {
  overflow: hidden;
  background-color: #e9e9e9;
}

.topnav a {
  float: left;
  display: block;
  color: black;
  text-align: center;
  padding: 14px 16px;
  text-decoration: none;
  font-size: 17px;
}

.topnav a:hover {
  background-color: #ddd;
  color: black;
}

.topnav a.active {
  background-color: #2196F3;
  color: white;
}

.topnav .search-container {
  float: right;
}

.topnav input[type=text] {
  padding: 6px;
  margin-top: 8px;
  font-size: 17px;
  border: none;
}

.topnav .search-container button {
  float: right;
  padding: 6px 10px;
  margin-top: 8px;
  margin-right: 16px;
  background: #ddd;
  font-size: 17px;
  border: none;
  cursor: pointer;
}

.topnav .search-container button:hover {
  background: #ccc;
}

@media screen and (max-width: 600px) {
  .topnav .search-container {
    float: none;
  }
  .topnav a, .topnav input[type=text], .topnav .search-container button {
    float: none;
    display: block;
    text-align: left;
    width: 100%;
    margin: 0;
    padding: 14px;
  }
  .topnav input[type=text] {
    border: 1px solid #ccc;
  }
}
</style>

  <div class="search-container">
    <form action="/action_page.php">
      <input type="text" placeholder="Search.." name="search">
      <button type="submit"><i class="fa fa-search"></i></button>
    </form>
  </div>

*/
