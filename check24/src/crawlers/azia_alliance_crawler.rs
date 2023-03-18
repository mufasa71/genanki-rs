use crate::crawlers::{Credit, text_helpers::*};
use scraper::{Html, Selector};

const URL: &str = "https://aab.uz/ru/private/crediting/";

pub async fn asia_alliance_parser(url: Option<&str>) -> Result<Vec<Credit>, Box<dyn std::error::Error>> {
    let url = url.unwrap_or(URL);
    let html = reqwest::get(url)
        .await?
        .text()
        .await?;

    let document = Html::parse_document(&html);
    let credit_selector = Selector::parse(".element--crediting").unwrap();
    let title_selector = Selector::parse(".element__title").unwrap();
    let params_selector = Selector::parse(".element__params .element__param").unwrap();
    let mut credits: Vec<Credit> = vec![];

    for element in document.select(&credit_selector) {
        let title = clean_text(element.select(&title_selector).next().unwrap().text());
        let mut credit = Credit::new();
        credit.title = title;
        credit.credit_type = find_credit_type(&credit.title);

        element
            .select(&params_selector)
            .enumerate()
            .for_each(|(i, element)| {
                let value = clean_text(element.text());

                match i {
                    0 => credit.interest_rate = value,
                    1 => credit.credit_period = value,
                    2 => credit.max_sum = value,
                    _ => (),
                }
            });

        credits.push(credit);
    }

    Ok(credits)
}

