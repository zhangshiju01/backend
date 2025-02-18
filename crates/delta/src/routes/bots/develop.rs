use revolt_quark::variables::delta::BOT_SERVER_PUBLIC_URL;
use revolt_result::{create_error, Result};
use rocket::serde::json::Json;
use serde_json::Value;

/// # develop Bot
///
/// develop a Revolt bot.
#[openapi(tag = "Bots")]
#[post("/develop/general/<target>", data = "<data>")]
pub async fn develop_bot(target: String, data: Json<Value>) -> Result<String> {
    info!("target: {}", target);
    info!("data: {:?}", data);

    let response = develop_bot_at_bot_server(target, data).await;
    Ok(response?)
}

async fn develop_bot_at_bot_server(target: String, data: Json<Value>) -> Result<String> {
    let host = BOT_SERVER_PUBLIC_URL.to_string();
    let url = format!("{host}/api/rest/v1/bot/develop/{target}");

    let client = reqwest::Client::new();
    let response = client
        .post(url.clone())
        .json(&*data)
        .send()
        .await
        .map_err(|_| create_error!(InternalError))?
        .text()
        .await
        .map_err(|_| create_error!(InternalError))?;

    Ok(response.to_string())
}

#[cfg(test)]
mod test {
    use crate::{rocket, util::test::TestHarness};
    use rocket::http::{ContentType, Header, Status};

    #[rocket::async_test]
    async fn develop_bot() {
        let harness = TestHarness::new().await;
        let (_, session, _) = harness.new_user().await;

        let response = harness
            .client
            .post("/bots/develop/general/generateWelcomeMessage")
            .header(Header::new("x-session-token", session.token.to_string()))
            .header(ContentType::JSON)
            .body(json!({}).to_string())
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
    }
}
