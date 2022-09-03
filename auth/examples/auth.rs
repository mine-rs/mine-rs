use miners_auth as auth;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let device_code = auth::DeviceCode::new("389b1b32-b5d5-43b2-bddc-84ce938d6737", None, &client)
        .await
        .unwrap();

    if let Some(inner) = &device_code.inner {
        println!("{}", inner.message)
    }

    //let code_resp = client
    //    .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
    //    .form(&[
    //        ("client_id", "389b1b32-b5d5-43b2-bddc-84ce938d6737"),
    //        ("scope", "XboxLive.signin offline_access"),
    //        ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
    //        ("device_code", "DEVICECODE"),
    //    ]).build()?;
    //let body = std::str::from_utf8(code_resp.body().unwrap().as_bytes().unwrap()).unwrap();
    //dbg!( body);

    let mca = device_code.authenticate(&client).await.unwrap();
    println!("{}", mca.name);
    Ok(())
}
