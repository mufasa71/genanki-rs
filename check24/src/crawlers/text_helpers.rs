use scraper::element_ref::Text;

pub fn clean_text(text: Text) -> String {
    text.map(|x| x.trim())
        .filter(|x| !x.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}
