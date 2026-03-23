pub fn root_page() -> &'static str {
    include_str!("../web/index.html")
}

pub fn app_css() -> &'static str {
    include_str!("../web/app.css")
}

pub fn app_js() -> &'static str {
    include_str!("../web/app.js")
}

pub fn languages_xml() -> &'static str {
    include_str!("../resources/languages.xml")
}
