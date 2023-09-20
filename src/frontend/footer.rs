pub fn generate_footer_html() -> String {
    let mut footer = String::new();
    footer.push_str("<footer>");
    footer.push_str(
        r#"
    <a href="https://paypal.me/Cravail" target=\"_blank\">
        <img src="assets/bmc.png" alt="fail" width="160" height="60" />
    </a>
"#,
    );
    footer.push_str(
        r#"
    <a href="https://github.com/halver94/Scrapy" target=\"_blank\">
        <img src="assets/github.jpg" alt="fail" width="160" height="60" />
    </a>
"#,
    );
    footer.push_str("</footer>");
    footer
}
