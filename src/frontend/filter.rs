use serde::{Deserialize, Serialize};

use super::server::State;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Filters {
    pub city: Option<String>,
    pub name: Option<String>,
    pub pro: bool,
}

impl Filters {
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
