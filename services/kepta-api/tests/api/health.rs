use crate::helpers::spawn_app;

#[tokio::test]
async fn health_check_returns_200() {
    let app = spawn_app().await;
    let res = app.health().await;
    assert_eq!(res.status().as_u16(), 200);
    assert_eq!(res.content_length(), Some(0));
}
