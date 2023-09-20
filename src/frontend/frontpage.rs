use crate::game::Games;

use super::server::{format_url_params, State};

pub fn create_html_table(games: Games, state: &mut State) -> String {
    let mut init_state = state.clone();
    let mut table = String::new();
    table.push_str(
        "<style>
                    table {
                        border-collapse: collapse;
                        width: 100%;
                    }
                    th, td {
                        border: 1px solid black;
                        padding: 8px;
                        text-align: center;
                    }
                    th {
                        background-color: lightgray;
                    }
                   
                    .flex-container {
                        display: flex;
                        align-items: center; /* Alignement vertical */
                    }
                    .flex-container img {
                        margin-right: 10px; /* Espacement entre l'image et le texte */
                    }

                    </style>",
    );
    table.push_str("<table>");
    table.push_str(
        format!(
            "{}{}{}{}{}{}{}",
            r#"<tr>
            <th>Updated <button onclick="window.location.href='/"#,
            {
                init_state.sort.sort = "updated".to_string();
                format_url_params(&init_state)
            },
            r#"';">Sort</button></th>
            <th>Name</th>
            <th>City</th>
            <th>Seller</th>
            <th>Shipping</th>
            <th>Deal <button onclick="window.location.href='/"#,
            {
                init_state.sort.sort = "price".to_string();
                format_url_params(&init_state)
            },
            r#"';">Sort €</button>
            <button onclick="window.location.href='/"#,
            {
                init_state.sort.sort = "percent".to_string();
                format_url_params(&init_state)
            },
            r#"';">Sort %</button></th>
            <th>Okkazeo</th>
            <th>Shops</th>
            <th>Note</th>
        </tr>"#
        )
        .as_str(),
    );

    for game in games.games.iter() {
        table.push_str("<tr>");
        table.push_str(&format!(
            "<td>{}</td>",
            game.okkazeo_announce
                .last_modification_date
                .format("%d/%m/%Y %H:%M")
        ));
        table.push_str(&format!(
            "<td>
                    <div class=\"flex-container\">
                        <img src=\"{}\" alt=\"fail\" width=\"100\" height=\"100\" />
                        {}<br>({})
                    </div>
                </td>",
            game.okkazeo_announce.image,
            game.okkazeo_announce.name,
            game.okkazeo_announce.extension
        ));
        table.push_str(&format!(
            "<td>{}</td>",
            game.okkazeo_announce.city.clone().unwrap_or(String::new())
        ));
        table.push_str(&format!(
            "<td>
                <a href=\"{}\" target=\"_blank\">{} {}
                    <a href=\'/{}\' target=\"_blank\">
                        <img src=\"assets/filter.png\" alt=\"fail\" width=\"20\" height=\"20\" />
                    </a>
                <br>({} announces)
                </a>
            </td>",
            game.okkazeo_announce.seller.url,
            game.okkazeo_announce.seller.name,
            if game.okkazeo_announce.seller.is_pro {
                "- PRO"
            } else {
                ""
            },
            {
                init_state.filters.vendor = Some(game.okkazeo_announce.seller.name.clone());
                format_url_params(&init_state)
            },
            game.okkazeo_announce.seller.nb_announces
        ));

        table.push_str("<td>");
        for (key, val) in game.okkazeo_announce.shipping.iter() {
            table.push_str(&format!("- {} : {}€<br>", key, val));
        }
        table.push_str("</td>");

        if game.deal.deal_price != 0 {
            table.push_str(&format!(
                "<td style=\"color: {}\">{}{}€ ({}{}%)</td>",
                if game.deal.deal_price < 0 {
                    "green"
                } else {
                    "red"
                },
                if game.deal.deal_price >= 0 { "+" } else { "" },
                game.deal.deal_price,
                if game.deal.deal_percentage > 0 {
                    "+"
                } else {
                    ""
                },
                game.deal.deal_percentage,
            ));
        } else {
            table.push_str("<td>-</td>");
        }

        table.push_str(&format!(
            "<td><a href=\"{}\" target=\"_blank\">{} &euro;</a></td>",
            game.okkazeo_announce.url, game.okkazeo_announce.price,
        ));

        table.push_str("<td>");
        if game.references.is_empty() {
            table.push_str("-");
        } else {
            for val in game.references.values() {
                table.push_str(&format!(
                    "<a href=\"{}\" target=\"_blank\">{} : {} &euro;</a><br>",
                    val.url, val.name, val.price,
                ));
            }
        }
        table.push_str("</td>");

        if game.review.average_note == 0.0 {
            table.push_str("<td>-</td>");
        } else {
            table.push_str(&format!(
                "<td>Average note : {:.2}<br><br>",
                game.review.average_note,
            ));

            for val in game.review.reviews.values() {
                table.push_str(&format!(
                    "<a href=\"{}\" target=\"_blank\">{}: {} ({} reviews)</a><br>",
                    val.url, val.name, val.note, val.number
                ));
            }
            table.push_str("</td>");
        }
        table.push_str("</tr>");
    }

    table.push_str("</table>");
    table
}
