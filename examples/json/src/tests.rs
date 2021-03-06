use rocket;
use rocket::testing::MockRequest;
use rocket::http::Method::*;
use rocket::http::{Status, ContentType};
use rocket::Response;

macro_rules! run_test {
    ($req:expr, $test_fn:expr) => ({
        let rocket = rocket::ignite()
            .mount("/message", routes![super::new, super::update, super::get])
            .catch(errors![super::not_found]);

        $test_fn($req.dispatch_with(&rocket));
    })
}

#[test]
fn bad_get_put() {
    // Try to get a message with an ID that doesn't exist.
    let mut req = MockRequest::new(Get, "/message/99").header(ContentType::JSON);
    run_test!(req, |mut response: Response| {
        assert_eq!(response.status(), Status::NotFound);

        let body = response.body().unwrap().into_string().unwrap();
        assert!(body.contains("error"));
        assert!(body.contains("Resource was not found."));
    });

    // Try to get a message with an invalid ID.
    let mut req = MockRequest::new(Get, "/message/hi").header(ContentType::JSON);
    run_test!(req, |mut response: Response| {
        assert_eq!(response.status(), Status::NotFound);
        let body = response.body().unwrap().into_string().unwrap();
        assert!(body.contains("error"));
    });

    // Try to put a message without a proper body.
    let mut req = MockRequest::new(Put, "/message/80").header(ContentType::JSON);
    run_test!(req, |response: Response| {
        assert_eq!(response.status(), Status::BadRequest);
    });

    // Try to put a message for an ID that doesn't exist.
    let mut req = MockRequest::new(Put, "/message/80")
        .header(ContentType::JSON)
        .body(r#"{ "contents": "Bye bye, world!" }"#);

    run_test!(req, |response: Response| {
        assert_eq!(response.status(), Status::NotFound);
    });
}

#[test]
fn post_get_put_get() {
    // Check that a message with ID 1 doesn't exist.
    let mut req = MockRequest::new(Get, "/message/1").header(ContentType::JSON);
    run_test!(req, |response: Response| {
        assert_eq!(response.status(), Status::NotFound);
    });

    // Add a new message with ID 1.
    let mut req = MockRequest::new(Post, "/message/1")
        .header(ContentType::JSON)
        .body(r#"{ "contents": "Hello, world!" }"#);

    run_test!(req, |response: Response| {
        assert_eq!(response.status(), Status::Ok);
    });

    // Check that the message exists with the correct contents.
    let mut req = MockRequest::new(Get, "/message/1") .header(ContentType::JSON);
    run_test!(req, |mut response: Response| {
        assert_eq!(response.status(), Status::Ok);
        let body = response.body().unwrap().into_string().unwrap();
        assert!(body.contains("Hello, world!"));
    });

    // Change the message contents.
    let mut req = MockRequest::new(Put, "/message/1")
        .header(ContentType::JSON)
        .body(r#"{ "contents": "Bye bye, world!" }"#);

    run_test!(req, |response: Response| {
        assert_eq!(response.status(), Status::Ok);
    });

    // Check that the message exists with the updated contents.
    let mut req = MockRequest::new(Get, "/message/1") .header(ContentType::JSON);
    run_test!(req, |mut response: Response| {
        assert_eq!(response.status(), Status::Ok);
        let body = response.body().unwrap().into_string().unwrap();
        assert!(!body.contains("Hello, world!"));
        assert!(body.contains("Bye bye, world!"));
    });
}
