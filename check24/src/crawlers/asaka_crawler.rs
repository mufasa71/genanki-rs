use crate::crawlers::{Credit, text_helpers::clean_text};
use scraper::{Html, Selector};

pub async fn asaka_parser() -> Result<(), Box<dyn std::error::Error>> {
    let json = reqwest::get("https://back.asakabank.uz/1/credit/?category=5&page_size=50")
        .await?
        .json::<serde_json::Value>()
        .await?;

    let mut credits: Vec<Credit> = vec![];
    for item in json["results"].as_array().unwrap() {
        let mut credit = Credit {
            title: String::new(),
            rate: String::new(),
            term: String::new(),
            sum: String::new(),
        };
        credit.title = item["title_ru"].as_str().unwrap().to_string();
        let sum_html = Html::parse_fragment(item["max_amount_ru"].as_str().unwrap());
        let sum_selector_p = Selector::parse("p").unwrap();
        let sum_selector_h3 = Selector::parse("h3").unwrap();

        match sum_html.select(&sum_selector_p).next() {
            Some(element) => credit.sum = clean_text(element.text()),
            None => (),
        }

        match sum_html.select(&sum_selector_h3).next() {
            Some(element) => credit.sum = clean_text(element.text()),
            None => (),
        }

        let term_html = Html::parse_fragment(item["credit_period_ru"].as_str().unwrap());
        let term_selector_h1 = Selector::parse("h1").unwrap();

        match term_html.select(&term_selector_h1).next() {
            Some(element) => credit.term = clean_text(element.text()),
            None => (),
        }

        credits.push(credit);
    }

    println!("{:?}", credits);
    Ok(())
}
