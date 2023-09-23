pub fn generate_header_html() -> String {
    let mut footer = String::new();
    footer.push_str("<header>");
    footer.push_str(
        r#"
        <style>
            #wrapper {
            width: 100%;
            overflow: hidden;
            }
            #container {
            width: 100%;
            margin: 0 auto;
            }
            .banner-img {
            width: 70%;
            display: block;
            margin: auto;
            }
        </style>
    "#,
    );
    footer.push_str(
        r#"
            <div id="banner">
                <div id="wrapper">
                    <div id="container">
                        <a><img class="banner-img" src="assets/banner.jpg" alt="fail"></a>
                    </div>
                </div>
            </div>
    "#,
    );
    footer.push_str("</header>");
    footer
}
