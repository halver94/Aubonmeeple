pub fn generate_footer_html() -> String {
    let html_str = r#"
    <a href="https://paypal.me/Cravail">
        <img src="assets/bmc.png" alt="fail" width="160" height="60" />
    </a>
"#;
    html_str.to_string()
}
