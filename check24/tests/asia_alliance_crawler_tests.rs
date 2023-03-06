use std::{fs::File, io::Read};

use check24uz::crawlers::{asia_alliance_parser, Credit};

#[tokio::test]
async fn asaka_crawler_tests() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    let mut credits_html = File::open("tests/stubs/asia_alliance/asia_alliance.html").unwrap();
    let mut contents = String::new();
    credits_html.read_to_string(&mut contents).unwrap();

    let m1 = server
        .mock("GET", mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body(contents)
        .create_async()
        .await;

    let credits = asia_alliance_parser(Some(&url)).await;

    match credits {
        Ok(credits) => {
            let result: Vec<Credit> = serde_json::from_reader(
                File::open("tests/stubs/asia_alliance/asia-alliance-result.json").unwrap(),
            )
            .unwrap();

            result.iter().enumerate().for_each(|(i, credit)| {
                assert_eq!(
                    credit.title, credits[i].title,
                    "Title is not equal for credit: {}",
                    credit.title
                );
                assert_eq!(
                    credit.rate, credits[i].rate,
                    "Rate is not equal for credit: {}",
                    credit.title
                );
                assert_eq!(
                    credit.term, credits[i].term,
                    "Term is not equal for credit: {}",
                    credit.title
                );
                assert_eq!(
                    credit.sum, credits[i].sum,
                    "Sum is not equal for credit: {}",
                    credit.title
                );
                assert_eq!(
                    credit.credit_type, credits[i].credit_type,
                    "Credit type is not equal for credit: {}",
                    credit.title
                );
            });
        }
        Err(e) => panic!("Error: {}", e),
    }

    m1.assert_async().await;
}
