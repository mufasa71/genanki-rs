use std::fs::File;

use check24uz::crawlers::{asaka_parser, Credit};
use futures::join;
use serde_json::Value;

#[tokio::test]
async fn asaka_crawler_tests() {
    let mut server = mockito::Server::new_async().await;
    let url = server.url();
    let credits_json: Value =
        serde_json::from_reader(File::open("tests/stubs/asaka/asaka-credits-list.json").unwrap())
            .unwrap();

    let m1 = server
        .mock("GET", "/1/credit/?category=5&page_size=50")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(credits_json.to_string())
        .create_async();

    let m2 = server
        .mock(
            "GET",
            mockito::Matcher::Regex(r"^/1/credit/(4|7|8|33|60|61)/property/$".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body_from_request(|request| {
            let id = request.path().split('/').nth(3).unwrap();
            let property_json: Value = serde_json::from_reader(
                File::open(format!(
                    "tests/stubs/asaka/asaka-credits-property-{}.json",
                    id
                ))
                .unwrap(),
            )
            .unwrap();

            property_json.to_string().into()
        })
        .create_async();

    let (m1, m2) = join!(m1, m2);
    let credits = asaka_parser(Some(&url)).await;

    match credits {
        Ok(credits) => {
            let result: Vec<Credit> =
                serde_json::from_reader(File::open("tests/stubs/asaka/asaka-result.json").unwrap())
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
            });
        }
        Err(e) => panic!("Error: {}", e),
    }

    m1.assert_async().await;
    m2.expect_at_most(6).assert_async().await;
}
