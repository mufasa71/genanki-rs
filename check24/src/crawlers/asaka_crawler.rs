use crate::crawlers::{text_helpers::*, Credit};
use scraper::{Html, Selector};

const HOST: &str = "https://back.asakabank.uz";

pub async fn asaka_parser(url: Option<&str>) -> Result<Vec<Credit>, Box<dyn std::error::Error>> {
    let url = url.unwrap_or(HOST);
    let client = reqwest::Client::new();
    let json = client
        .get(format!("{}/1/credit/?category=5&page_size=50", url))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let mut credits: Vec<Credit> = vec![];
    for item in json["results"].as_array().unwrap() {
        let title = item["title_ru"].as_str().unwrap().to_string();
        let mut credit = Credit::new();
        credit.title = title;
        credit.credit_type = find_credit_type(&credit.title);
        let sum_html = Html::parse_fragment(item["max_amount_ru"].as_str().unwrap());
        let sum_selector_p = Selector::parse("p").unwrap();
        let sum_selector_h3 = Selector::parse("h3").unwrap();

        if let Some(element) = sum_html.select(&sum_selector_p).next() {
            credit.max_sum = clean_text(element.text())
        }

        if let Some(element) = sum_html.select(&sum_selector_h3).next() {
            credit.max_sum = clean_text(element.text())
        }

        let term_html = Html::parse_fragment(item["credit_period_ru"].as_str().unwrap());
        let term_selector_h1 = Selector::parse("h1").unwrap();

        if let Some(element) = term_html.select(&term_selector_h1).next() {
            credit.credit_period = clean_text(element.text())
        }

        let property = client
            .get(format!("{}/1/credit/{id}/property/", url, id = item["id"]))
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        credit.interest_rate = property["results"][0]["interest_rate"]
            .as_str()
            .unwrap()
            .to_string();
        credits.push(credit);
    }

    Ok(credits)
}
