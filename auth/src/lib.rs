#![deny(clippy::undocumented_unsafe_blocks)]
//TODO: Add documentation and fix naming

use {
    std::{string::FromUtf8Error, path::Path},
    async_fs as fs,
    reqwest::{Client, StatusCode},
    serde_derive::{Deserialize, Serialize},
    serde_json::json,
    futures_io::{AsyncRead, AsyncWrite,},
    futures_util::{AsyncReadExt, AsyncWriteExt}
};

const CACHE_FILE_NAME: &str = "auth.cache";

#[derive(Debug, thiserror::Error)]
pub enum HttpOrSerdeError {
    #[error("Something went wrong while sending a http request.\n{0}")]
    HttpError(#[from] reqwest::Error),
    #[error("Something went wrong while decoding the json response:\n{0}")]
    SerdeError(#[from] serde_json::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum ReadError {
    #[error("Something went wrong while reading from the file.\nError: {0}")]
    Io(#[from] std::io::Error),
    #[error("String data was invalid utf-8.\nError: {0}")]
    FromUtf8Error(#[from] FromUtf8Error),
}

async fn read_string_from<R: AsyncRead + Unpin>(r: &mut R) -> Result<String, ReadError> {
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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    async fn mc_profile(&self, client: &Client) -> Result<McProfile, HttpOrSerdeError> {
        let pr_resp = client
            .get("https://api.minecraftservices.com/minecraft/profile")
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;

        let mc_profile = serde_json::from_slice(&pr_resp)?;
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
    async fn auth_mc(&self, client: &Client) -> Result<McAuth, HttpOrSerdeError> {
        let json = json!({
            "identityToken": format!("XBL3.0 x={};{}", self.display_claims.xui[0].uhs, self.token)
        });

        let mc_resp = client
            .post("https://api.minecraftservices.com/authentication/login_with_xbox")
            .header("Accept", "application/json")
            .json(&json)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;

        let mc_auth: McAuth = serde_json::from_slice(&mc_resp)?;
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
    async fn auth_xsts(&self, client: &Client) -> Result<XstsAuth, HttpOrSerdeError> {
        let json = json!({
            "Properties": {
                "SandboxId":  "RETAIL",
                "UserTokens": [self.token]
            },
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType":    "JWT",
        });

        let xsts_resp = client
            .post("https://xsts.auth.xboxlive.com/xsts/authorize")
            .header("Content-Type", "application/json")
            .json(&json)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;

        let xsts_auth: XstsAuth = serde_json::from_slice(&xsts_resp)?;

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
    pub async fn refresh(&mut self, cid: &str, client: &Client) -> Result<bool, HttpOrSerdeError> {
        if self.expires_after <= chrono::Utc::now().timestamp() {
            let resp = client
                .post("https://login.live.com/oauth20_token.srf")
                .form(&[
                    ("client_id", cid),
                    ("refresh_token", &self.refresh_token),
                    ("grant_type", "refresh_token"),
                    (
                        "redirect_uri",
                        "https://login.microsoftonline.com/common/oauth2/nativeclient",
                    ),
                ])
                .send()
                .await?
                .error_for_status()?
                .bytes()
                .await?;
            let refresh: MsAuthRefresh = serde_json::from_slice(&resp)?;
            self.access_token = refresh.access_token;
            self.refresh_token = refresh.refresh_token;
            self.expires_after = refresh.expires_in + chrono::Utc::now().timestamp();
            return Ok(true);
        }
        Ok(false)
    }

    pub async fn auth_xbl(&self, client: &Client) -> Result<XblAuth, HttpOrSerdeError> {
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
            .post("https://user.auth.xboxlive.com/user/authenticate")
            .header("Accept", "application/json")
            .json(&json)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;
        let xbl_auth: XblAuth = serde_json::from_slice(&xbl_resp)?;
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

    pub async fn read_from<R: AsyncRead + Unpin>(r: &mut R) -> Result<MsAuth, ReadError> {
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

#[derive(Default, Debug, Clone, PartialEq)]
pub struct DeviceCode {
    pub inner: Option<DeviceCodeInner>,
    cid: String,
    cache: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeviceCodeInner {
    pub user_code: String,
    device_code: String,
    pub verification_uri: String,
    expires_in: i64,
    interval: u64,
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum HttpOrSerdeOrMsAuthError {
    #[error("{0}")]
    MsAuthError(#[from] MsAuthError),
    #[error("Something went wrong while sending a http request.\nError: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("Something went wrong while decoding the json response:\nError: {0}")]
    SerdeError(#[from] serde_json::Error)
}

#[derive(Debug, thiserror::Error)]
pub enum AuthenticateError {
    #[error("{0}")]
    HttpOrSerdeError(#[from] HttpOrSerdeError),
    #[error("{0}")]
    IoError(#[from] std::io::Error),
    #[error("{0}")]
    MsauthError(#[from] HttpOrSerdeOrMsAuthError),
    #[error("{0}")]
    ReadError(#[from] ReadError)
}

impl DeviceCode {
    /// Entry point of the auth flow.
    /// It's up to you how you show the user the user code and the link
    /// Only show the user code and the link when cached is false because they'll be empty if not.
    pub async fn new(
        cid: &str,
        cache_file: Option<&str>,
        client: &Client,
    ) -> Result<Self, HttpOrSerdeError> {
        let (path, name) = match cache_file {
            Some(file) => (Path::new(file), file),
            None => (Path::new(CACHE_FILE_NAME), CACHE_FILE_NAME),
        };

        let device_code: DeviceCode;
        let device_code_inner: Option<DeviceCodeInner>;
        if !path.is_file() {
            let device_resp = client
                .get("https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode")
                .query(&[
                    ("client_id", cid),
                    ("scope", "XboxLive.signin offline_access"),
                ])
                .header("content-length", "0")
                .send()
                .await?
                .error_for_status()?
                .bytes()
                .await?;
            device_code_inner = Some(serde_json::from_slice(&device_resp)?);
        } else {
            device_code_inner = None;
        }
        device_code = DeviceCode {
            inner: device_code_inner,
            cid: String::from(cid),
            cache: String::from(name),
        };
        Ok(device_code)
    }

    async fn auth_ms(&self, client: &Client) -> Result<Option<MsAuth>, HttpOrSerdeOrMsAuthError> {
        match &self.inner {
            Some(inner) => loop {
                std::thread::sleep(std::time::Duration::from_secs(inner.interval + 1));

                let code_resp = client
                    .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
                    .form(&[
                        ("client_id", &self.cid as &str),
                        ("scope", "XboxLive.signin offline_access"),
                        ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                        ("device_code", &inner.device_code),
                    ])
                    .send()
                    .await?;
                match code_resp.status() {
                    StatusCode::BAD_REQUEST => {
                        let ms_auth_error: MsAuthError = serde_json::from_slice(&code_resp.bytes().await?)?;
                        match &ms_auth_error.error as &str {
                            "authorization_pending" => continue,
                            _ => return Err(ms_auth_error.into())
                        }
                    }
                    StatusCode::OK => {
                        let mut ms_auth: MsAuth = serde_json::from_slice(&code_resp.bytes().await?)?;
                        ms_auth.expires_after = ms_auth.expires_in + chrono::Utc::now().timestamp();
                        return Ok(Some(ms_auth));
                    }
                    _ => {
                        code_resp.error_for_status()?;
                    }
                }
            },
            None => Ok(None),
        }
    }

    /// Call this method after creating the device code and having shown the user the code (but only if DeviceCode.cached is false)
    /// It might block for a while if the access token hasn't been cached yet.
    pub async fn authenticate(&self, client: &Client) -> Result<Auth, AuthenticateError> {
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
        let mca = msa.auth_xbl(client).await?.auth_xsts(client).await?.auth_mc(client).await?;

        let profile = mca.mc_profile(client).await?;

        let auth = Auth {
            name: profile.name,
            uuid: profile.id,
            token: mca.access_token,
        };
        Ok(auth)
    }
}
