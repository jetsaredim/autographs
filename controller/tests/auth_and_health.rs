use autographs_controller::{config::ControllerConfig, routes::router};
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use serde_json::Value;
use tower::ServiceExt;

#[tokio::test]
async fn auth_and_health_routes_are_redacted_and_guarded() {
    let app = router(ControllerConfig::for_test(true));

    let health = app
        .clone()
        .oneshot(Request::get("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(health.status(), StatusCode::OK);
    let body: Value =
        serde_json::from_slice(&to_bytes(health.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(body["service"], "autographs-controller");

    let admin_health = app
        .clone()
        .oneshot(
            Request::get("/admin/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(admin_health.status(), StatusCode::OK);
    let body = String::from_utf8(
        to_bytes(admin_health.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    for denied in [
        "ORACLE",
        "OCI_",
        "bucketName",
        "objectKey",
        "local-test-password",
        "operator-test-token",
    ] {
        assert!(!body.contains(denied), "health leaked {denied}");
    }

    let protected = app
        .clone()
        .oneshot(
            Request::get("/admin/api/protected")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(protected.status(), StatusCode::UNAUTHORIZED);

    let bearer = app
        .clone()
        .oneshot(
            Request::get("/admin/api/protected")
                .header(header::AUTHORIZATION, "Bearer operator-test-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(bearer.status(), StatusCode::OK);
}

#[tokio::test]
async fn auth_and_health_login_issues_strict_secure_cookie_and_cookie_mutations_require_same_origin()
 {
    let app = router(ControllerConfig::for_test(true));

    let invalid = login(&app, "wrong").await;
    assert_eq!(invalid.status(), StatusCode::UNAUTHORIZED);
    assert!(invalid.headers().get(header::SET_COOKIE).is_none());

    let valid = login(&app, "local-test-password").await;
    assert_eq!(valid.status(), StatusCode::NO_CONTENT);
    let set_cookie = valid
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(set_cookie.contains("autographs_admin_session="));
    assert!(set_cookie.contains("HttpOnly"));
    assert!(set_cookie.contains("SameSite=Strict"));
    assert!(set_cookie.contains("Secure"));
    let cookie = set_cookie.split(';').next().unwrap();

    let protected = app
        .clone()
        .oneshot(
            Request::get("/admin/api/protected")
                .header(header::COOKIE, cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(protected.status(), StatusCode::OK);

    let cross_origin = app
        .clone()
        .oneshot(
            Request::post("/admin/api/test-mutation")
                .header(header::COOKIE, cookie)
                .header(header::ORIGIN, "https://attacker.example")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cross_origin.status(), StatusCode::FORBIDDEN);

    let same_origin = app
        .clone()
        .oneshot(
            Request::post("/admin/api/test-mutation")
                .header(header::COOKIE, cookie)
                .header(header::ORIGIN, "https://autographs.example.test")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(same_origin.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn auth_and_health_insecure_cookie_mode_is_explicit_for_local_development() {
    let app = router(ControllerConfig::for_test(false));
    let valid = login(&app, "local-test-password").await;
    let set_cookie = valid
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap();

    assert!(!set_cookie.contains("Secure"));
}

async fn login(app: &axum::Router, password: &str) -> axum::response::Response {
    app.clone()
        .oneshot(
            Request::post("/admin/api/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(format!(r#"{{"password":"{password}"}}"#)))
                .unwrap(),
        )
        .await
        .unwrap()
}
