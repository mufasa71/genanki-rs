use scraper::{element_ref::Text, Html, Selector};

#[derive(Debug)]
struct Credit {
    title: String,
    rate: String,
    term: String,
    sum: String,
}

fn clean_text(text: Text) -> String {
    text.map(|x| x.trim())
        .filter(|x| !x.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

async fn asaka_parser() -> Result<(), Box<dyn std::error::Error>> {
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

async fn aab_parser() -> Result<(), Box<dyn std::error::Error>> {
    let html = reqwest::get("https://aab.uz/ru/private/crediting/")
        .await?
        .text()
        .await?;

    let document = Html::parse_document(&html);
    let credit_selector = Selector::parse(".element--crediting").unwrap();
    let title_selector = Selector::parse(".element__title").unwrap();
    let params_selector = Selector::parse(".element__params .element__param").unwrap();
    let mut credits: Vec<Credit> = vec![];

    for element in document.select(&credit_selector) {
        let mut credit = Credit {
            title: String::new(),
            rate: String::new(),
            term: String::new(),
            sum: String::new(),
        };

        credit.title = clean_text(element.select(&title_selector).next().unwrap().text());

        element
            .select(&params_selector)
            .enumerate()
            .for_each(|(i, element)| {
                let value = clean_text(element.text());

                match i {
                    0 => credit.rate = value,
                    1 => credit.term = value,
                    2 => credit.sum = value,
                    _ => (),
                }
            });

        credits.push(credit);
    }

    println!("{:?}", credits);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    aab_parser().await?;
    asaka_parser().await?;

    Ok(())
}
