use serde::{Deserialize, Serialize};

use super::server::{format_url_params, State};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Filters {
    pub city: Option<String>,
    pub name: Option<String>,
    pub vendor: Option<String>,
    pub pro: Option<bool>,
    pub note: Option<f32>,
    pub max_price: Option<i32>,
    pub min_price: Option<i32>,
}

// this is ugly, but otherwise the Form from axum doesnt work properly
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FiltersForm {
    pub city_form: Option<String>,
    pub name_form: Option<String>,
    pub vendor_form: Option<String>,
    pub pro_form: Option<String>,
    pub note_form: Option<String>,
    pub max_price_form: Option<String>,
    pub min_price_form: Option<String>,
}

impl Filters {
    pub fn create_html(state: &State) -> String {
        let state2 = state.clone();
        let params = format_url_params(&state2);

        let style = "
        <style>
            /* Style pour le conteneur du formulaire */
            .form-container {
                text-align: center;
                max-width: 400px;
                margin: 0 auto;
            }

            /* Style pour le formulaire */
            form {
                display: flex;
                flex-direction: row;
                align-items: left;
            }   

            /* Style pour chaque groupe label + input */
            .form-group {
                display: flex;
                flex-direction: column;
                align-items: left;
                margin-bottom: 15px;
                margin-right: 15px;
            }
        </style>";

        let html = format!("{}
            <div>
            <form action=\"/{}\" method=\"post\">
            <div class=\"form-group\"><label for=\"city\">Filter on city :</label><input type=\"text\" id=\"city\" name=\"city_form\" {}></div>
            <div class=\"form-group\"><label for=\"name\">Filter on name :</label><input type=\"text\" id=\"name\" name=\"name_form\" {}></div>
            <div class=\"form-group\"><label for=\"vendor\">Filter on vendor :</label><input type=\"text\" id=\"vendor\" name=\"vendor_form\" {}></div>
            <div class=\"form-group\"><label for=\"note\">Minimal note :</label><input type=\"number\" step=\"any\" id=\"note\" name=\"note_form\" {} min=\"0\" max=\"10\"></div>
            <div class=\"form-group\"><label for=\"min_price\">Minimal price :</label><input type=\"number\" step=\"1\" id=\"min_price\" name=\"min_price_form\" {} min=\"0\"></div>
            <div class=\"form-group\"><label for=\"max_price\">Maximal price :</label><input type=\"number\" step=\"1\" id=\"max_price\" name=\"max_price_form\" {}></div>
            <div class=\"form-group\"><label for=\"pro\">Exclude pro vendors</label><input type=\"checkbox\" id=\"pro\" name=\"pro_form\" {}></div>
                    <input type=\"submit\" value=\"Filter\">
                </form>
            </div>",
            style,
            params,
        if state2.filters.city.is_some() {format!("value=\"{}\"", state2.filters.city.unwrap())} else {"".to_string()},
        if state2.filters.name.is_some() {format!("value=\"{}\"", state2.filters.name.unwrap())} else {"".to_string()},
        if state2.filters.vendor.is_some() {format!("value=\"{}\"", state2.filters.vendor.unwrap())} else {"".to_string()},
        if state2.filters.note.is_some() {format!("value=\"{}\"", state2.filters.note.unwrap())} else {"".to_string()},
        if state2.filters.min_price.is_some() {format!("value=\"{}\"", state2.filters.min_price.unwrap())} else {"".to_string()},
        if state2.filters.max_price.is_some() {format!("value=\"{}\"", state2.filters.max_price.unwrap())} else {"".to_string()},
        if state2.filters.pro.is_some() {"checked"} else {""},
        );
        html
    }
}
