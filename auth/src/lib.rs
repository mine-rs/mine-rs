#![deny(clippy::undocumented_unsafe_blocks)]
//TODO: Add documentation and fix naming

use {
    async_fs as fs,
    async_trait::async_trait,
    futures_io::{AsyncRead, AsyncWrite},
    futures_util::{AsyncReadExt, AsyncWriteExt},
    http::StatusCode,
    serde_derive::{Deserialize, Serialize},
    serde_json::json,
    std::{fmt::Display, path::Path, string::FromUtf8Error},
};

trait ResponseExt: Sized {
    fn error_for_status(self) -> Result<Self, Error>;
}

impl<T> ResponseExt for http::Response<T> {
    fn error_for_status(self) -> Result<Self, Error> {
        let status = self.status();
        if !status.is_success() {
            Err(HttpStatusError::from(status).into())
        } else {
            Ok(self)
        }
    }
}

#[async_trait]
pub trait HttpClient {
    type Body: AsRef<[u8]>;
    async fn execute_request(
        &self,
        req: http::Request<Vec<u8>>,
    ) -> anyhow::Result<http::response::Response<Self::Body>>;
}

#[cfg(feature = "reqwest")]
#[async_trait]
impl HttpClient for reqwest::Client {
    type Body = bytes::Bytes;

    async fn execute_request(
        &self,
        req: http::Request<Vec<u8>>,
    ) -> anyhow::Result<http::response::Response<Self::Body>> {
        let req: reqwest::Request = req.try_into()?;
        //if let Some(b) = req.body() {
        //    dbg!(std::str::from_utf8(b.as_bytes().unwrap()).unwrap());
        //}
        let resp = self.execute(req).await?;
        let status = resp.status();
        let bytes = resp.bytes().await?;
        let mut http_resp = http::Response::new(bytes);
        let status2 = http_resp.status_mut();
        *status2 = status;
        Ok(http_resp)
    }
}

const CACHE_FILE_NAME: &str = "auth.cache";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    String(#[from] FromUtf8Error),
    #[error(transparent)]
    MsAuth(#[from] MsAuthError),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
    #[error(transparent)]
    HttpStatus(#[from] HttpStatusError),
    #[error(transparent)]
    Http(#[from] http::Error),
}

#[derive(Debug)]
pub struct HttpStatusError(StatusCode);

impl std::error::Error for HttpStatusError {}

impl Display for HttpStatusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<StatusCode> for HttpStatusError {
    fn from(s: StatusCode) -> Self {
        Self(s)
    }
}

async fn read_string_from<R: AsyncRead + Unpin>(r: &mut R) -> Result<String, Error> {
    let mut bytes = [0u8; 2];
    r.read_exact(&mut bytes).await?;
    let len = u16::from_le_bytes(bytes);
    let mut buf = vec![0; len as usize];
    r.read_exact(&mut buf).await?;
    Ok(String::from_utf8(buf)?)
}

async fn write_string_to<W: AsyncWrite + Unpin>(w: &mut W, s: &String) -> std::io::Result<()> {
    let len = (s.len() as u16).to_le_bytes();
    w.write_all(&len).await?;
    w.write_all(s.as_bytes()).await?;
    Ok(())
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Auth {
    pub name: String,
    pub uuid: String,
    pub token: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct McProfile {
    id: String,
    name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct McAuth {
    pub access_token: String,
    pub expires_in: i64,
    //#[serde(skip)]
    //pub expires_after: i64,
}

impl McAuth {
    async fn mc_profile(&self, client: &impl HttpClient) -> Result<McProfile, Error> {
        let pr_resp = client
            .execute_request(
                http::request::Builder::new()
                    .header("Authorization", format!("Bearer {}", self.access_token))
                    .body(Vec::new())?,
            )
            .await?
            .error_for_status()?
            .into_body();

        //let pr_resp = client
        //    .get("https://api.minecraftservices.com/minecraft/profile")
        //    .header("Authorization", format!("Bearer {}", self.access_token))
        //    .send()
        //    .await?
        //    .error_for_status()?
        //    .bytes()
        //    .await?;

        let mc_profile = serde_json::from_slice(pr_resp.as_ref())?;
        Ok(mc_profile)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct DisplayClaims {
    xui: Vec<Xui>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Xui {
    uhs: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct XstsAuth {
    token: String,
    display_claims: DisplayClaims,
}

impl XstsAuth {
    async fn auth_mc(&self, client: &impl HttpClient) -> Result<McAuth, Error> {
        let json = json!({
            "identityToken": format!("XBL3.0 x={};{}", self.display_claims.xui[0].uhs, self.token)
        });

        let mc_resp = client
            .execute_request(
                http::request::Builder::new()
                    .uri("https://api.minecraftservices.com/authentication/login_with_xbox")
                    .method(http::Method::POST)
                    .header("content-type", "application/json")
                    .body(serde_json::to_vec(&json)?)?,
            )
            .await?
            .error_for_status()?
            .into_body();

        //let mc_resp = client
        //    .post("https://api.minecraftservices.com/authentication/login_with_xbox")
        //    .header("Accept", "application/json")
        //    .json(&json)
        //    .send()
        //    .await?
        //    .error_for_status()?
        //    .bytes()
        //    .await?;

        let mc_auth: McAuth = serde_json::from_slice(mc_resp.as_ref())?;
        //mc_auth.expires_after = mc_auth.expires_in + chrono::Utc::now().timestamp();
        Ok(mc_auth)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct XblAuth {
    token: String,
}

impl XblAuth {
    async fn auth_xsts(&self, client: &impl HttpClient) -> Result<XstsAuth, Error> {
        let json = json!({
            "Properties": {
                "SandboxId":  "RETAIL",
                "UserTokens": [self.token]
            },
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType":    "JWT",
        });
        let xsts_resp = client
            .execute_request(
                http::request::Builder::new()
                    .uri("https://xsts.auth.xboxlive.com/xsts/authorize")
                    .method(http::Method::POST)
                    .header("content-type", "application/json")
                    .body(serde_json::to_vec(&json)?)?,
            )
            .await?
            .into_body();

        //let xsts_resp = client
        //    .post("https://xsts.auth.xboxlive.com/xsts/authorize")
        //    .header("Content-Type", "application/json")
        //    .json(&json)
        //    .send()
        //    .await?
        //    .error_for_status()?
        //    .bytes()
        //    .await?;

        let xsts_auth: XstsAuth = serde_json::from_slice(xsts_resp.as_ref())?;

        Ok(xsts_auth)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct MsAuthRefresh {
    expires_in: i64,
    access_token: String,
    refresh_token: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct MsAuth {
    expires_in: i64,
    access_token: String,
    refresh_token: String,
    #[serde(skip)]
    expires_after: i64,
}

impl MsAuth {
    /// Checks if the access token is still valid and refreshes it if it isn't.
    pub async fn refresh(&mut self, cid: &str, client: &impl HttpClient) -> Result<bool, Error> {
        if self.expires_after <= chrono::Utc::now().timestamp() {
            let resp = client.execute_request(
                http::request::Builder::new()
                .uri(
                    format!(
                        "https://login.live.com/oauth20_token.srf?client_id={}&refresh_token={}&grant_type={}&redirect_uri={}",
                        cid,
                        &self.refresh_token,
                        "refresh_token",
                        "https://login.microsoftonline.com/common/oauth2/nativeclient"
                    )
                )
                .method(http::Method::POST)
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Vec::new())?
            ).await?.into_body();
            //let resp = client
            //    .post("https://login.live.com/oauth20_token.srf")
            //    .form(&[
            //        ("client_id", cid),
            //        ("refresh_token", &self.refresh_token),
            //        ("grant_type", "refresh_token"),
            //        (
            //            "redirect_uri",
            //            "https://login.microsoftonline.com/common/oauth2/nativeclient",
            //        ),
            //    ])
            //    .send()
            //    .await?
            //    .error_for_status()?
            //    .bytes()
            //    .await?;
            let refresh: MsAuthRefresh = serde_json::from_slice(resp.as_ref())?;
            self.access_token = refresh.access_token;
            self.refresh_token = refresh.refresh_token;
            self.expires_after = refresh.expires_in + chrono::Utc::now().timestamp();
            return Ok(true);
        }
        Ok(false)
    }

    pub async fn auth_xbl(&self, client: &impl HttpClient) -> Result<XblAuth, Error> {
        let json = json!({
            "Properties": {
                "AuthMethod": "RPS",
                "SiteName":   "user.auth.xboxlive.com",
                "RpsTicket":  &(String::from("d=") + &self.access_token) as &str,
            },
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType":    "JWT",
        });

        let xbl_resp = client
            .execute_request(
                http::request::Builder::new()
                    .uri("https://user.auth.xboxlive.com/user/authenticate")
                    .method(http::Method::POST)
                    .body(serde_json::to_vec(&json)?)?,
            )
            .await?
            .into_body();
        //let xbl_resp = client
        //    .post("https://user.auth.xboxlive.com/user/authenticate")
        //    .header("Accept", "application/json")
        //    .json(&json)
        //    .send()
        //    .await?
        //    .error_for_status()?
        //    .bytes()
        //    .await?;
        let xbl_auth: XblAuth = serde_json::from_slice(xbl_resp.as_ref())?;
        Ok(xbl_auth)
    }

    pub async fn write_to<W: AsyncWrite + Unpin>(&self, w: &mut W) -> std::io::Result<()> {
        let mut buf = Vec::new();
        buf.write_all(&self.expires_after.to_le_bytes()).await?;
        write_string_to(&mut buf, &self.access_token).await?;
        write_string_to(&mut buf, &self.refresh_token).await?;
        let len = buf.len();
        w.write_all(&(len as u16).to_le_bytes()).await?;
        w.write_all(&buf).await?;
        Ok(())
    }

    pub async fn read_from<R: AsyncRead + Unpin>(r: &mut R) -> Result<MsAuth, Error> {
        let mut bytes = [0u8; 2];
        r.read_exact(&mut bytes).await?;
        let len = u16::from_le_bytes(bytes) as usize;

        let mut buf = vec![0; len];
        r.read_exact(&mut buf).await?;
        let mut buf = buf.as_slice();
        let mut bytes = [0u8; 8];
        buf.read_exact(&mut bytes).await?;

        let expires_after = i64::from_le_bytes(bytes);
        let access_token = read_string_from(&mut buf).await?;
        let refresh_token = read_string_from(&mut buf).await?;
        Ok(MsAuth {
            expires_in: 0,
            access_token,
            refresh_token,
            expires_after,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, thiserror::Error)]
#[error(
    "Something went wrong while requesting the Microsoft auth token.
Error: {error}
Description: {error_description}
Timestamp: {timestamp}
Correlation Id: {correlation_id}
Error URI: {error_uri}
"
)]
pub struct MsAuthError {
    pub error: String,
    pub error_description: String,
    pub error_codes: Vec<i64>,
    pub timestamp: String,
    pub trace_id: String,
    pub correlation_id: String,
    pub error_uri: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct DeviceCode {
    pub inner: Option<DeviceCodeInner>,
    cid: String,
    cache: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceCodeInner {
    pub user_code: String,
    pub device_code: String,
    pub verification_uri: String,
    expires_in: i64,
    interval: u64,
    pub message: String,
}

impl DeviceCode {
    /// Entry point of the auth flow.
    /// It's up to you how you show the user the user code and the link
    /// Only show the user code and the link when cached is false because they'll be empty if not.
    pub async fn new(
        cid: &str,
        cache_file: Option<&str>,
        client: &impl HttpClient,
    ) -> Result<Self, Error> {
        let (path, name) = match cache_file {
            Some(file) => (Path::new(file), file),
            None => (Path::new(CACHE_FILE_NAME), CACHE_FILE_NAME),
        };

        let device_code_inner: Option<DeviceCodeInner>;
        if !path.is_file() {
            let device_resp = client.execute_request(
                http::request::Builder::new()
                    .uri(
                        format!(
                            "https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode?client_id={cid}&scope={}",
                            "XboxLive.signin%20offline_access"
                        )
                    )
                    .header("content-length", "0")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Vec::new())?
                )
                .await?
                .error_for_status()?
                .into_body();

            //let device_resp = client
            //    .get("https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode")
            //    .query(&[
            //        ("client_id", cid),
            //        ("scope", "XboxLive.signin offline_access"),
            //    ])
            //    .header("content-length", "0")
            //    .send()
            //    .await?
            //    .error_for_status()?
            //    .bytes()
            //    .await?;
            device_code_inner = Some(serde_json::from_slice(device_resp.as_ref())?);
        } else {
            device_code_inner = None;
        }
        let device_code = DeviceCode {
            inner: device_code_inner,
            cid: String::from(cid),
            cache: String::from(name),
        };
        Ok(device_code)
    }

    async fn auth_ms(&self, client: &impl HttpClient) -> Result<Option<MsAuth>, Error> {
        match &self.inner {
            Some(inner) => {
                let mut interval =
                    async_timer::interval(std::time::Duration::from_secs(inner.interval + 1));
                loop {
                    interval.wait().await;
                    let body = format!(
                        "grant_type={}&client_id={}&device_code={}",
                        "urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Adevice_code",
                        &self.cid as &str,
                        &inner.device_code
                    )
                    .into_bytes();
                    let code_resp = client
                        .execute_request(
                            http::request::Builder::new()
                                .method(http::Method::POST)
                                .header("content-type", "application/x-www-form-urlencoded")
                                .header("content-length", body.len())
                                .uri(
                                    "https://login.microsoftonline.com/consumers/oauth2/v2.0/token",
                                )
                                .body(body)?,
                        )
                        .await?;
                    //let code_resp = client
                    //    .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
                    //    .form(&[
                    //        ("client_id", &self.cid as &str),
                    //        ("scope", "XboxLive.signin offline_access"),
                    //        ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                    //        ("device_code", &inner.device_code),
                    //    ])
                    //    .send()
                    //    .await?;
                    match code_resp.status() {
                        StatusCode::BAD_REQUEST => {
                            let ms_auth_error: MsAuthError =
                                serde_json::from_slice(code_resp.into_body().as_ref())?;
                            match &ms_auth_error.error as &str {
                                "authorization_pending" => continue,
                                _ => return Err(ms_auth_error.into()),
                            }
                        }
                        StatusCode::OK => {
                            let mut ms_auth: MsAuth =
                                serde_json::from_slice(code_resp.into_body().as_ref())?;
                            ms_auth.expires_after =
                                ms_auth.expires_in + chrono::Utc::now().timestamp();
                            return Ok(Some(ms_auth));
                        }
                        _ => {
                            code_resp.error_for_status()?;
                        }
                    }
                }
            }
            None => Ok(None),
        }
    }

    /// Call this method after creating the device code and having shown the user the code (but only if DeviceCode.cached is false)
    /// It might block for a while if the access token hasn't been cached yet.
    pub async fn authenticate(&self, client: &impl HttpClient) -> Result<Auth, Error> {
        let path: &Path = Path::new(&self.cache);
        let msa = match self.inner {
            Some(_) => {
                // SAFETY: Because we know self.inner is Some, we can be certain self.auth_ms() won't return None.
                let msa = unsafe { self.auth_ms(client).await?.unwrap_unchecked() };
                msa.write_to(&mut fs::File::create(path).await?).await?;
                msa
            }
            None => {
                let mut msa: MsAuth = MsAuth::read_from(&mut fs::File::open(path).await?).await?;
                if msa.refresh(&self.cid, client).await? {
                    msa.write_to(&mut fs::File::create(path).await?).await?;
                }
                msa
            }
        };
        let mca = msa
            .auth_xbl(client)
            .await?
            .auth_xsts(client)
            .await?
            .auth_mc(client)
            .await?;

        dbg!("hi");
        let profile = mca.mc_profile(client).await?;

        let auth = Auth {
            name: profile.name,
            uuid: profile.id,
            token: mca.access_token,
        };
        Ok(auth)
    }
}
