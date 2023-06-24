use std::io::Cursor;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, bail};
use async_std::io::ReadExt;
use async_std::net::TcpStream;
use async_std::sync::Mutex;
use async_std::task::{sleep, spawn};
use futures_lite::io::{BufReader, BufWriter};
use isahc::http::StatusCode;
use miners::encoding::attrs::StringUuid;
use miners::encoding::{Decode, Encode};
use miners::nbt;
use miners::protocol::netty::handshaking::serverbound::NextState0;
use miners::protocol::netty::handshaking::SbHandshaking;
use miners::protocol::netty::login::clientbound::{
    EncryptionRequest19, SetCompression27, Success0,
};
use miners::protocol::netty::login::serverbound::{EncryptionResponse19, LoginStart0};
use miners::protocol::netty::login::SbLogin;
use miners::protocol::netty::play::clientbound::{
    ChunkData27, Dimension0, GameMode0, JoinGame29, KeepAlive32, PlayerAbilities0,
    PositionAndLook6, SpawnPosition6,
};
use miners::protocol::netty::play::serverbound::KeepAlive7;
use miners::protocol::netty::play::SbPlay;
use miners::{
    net::encoding::Encoder, protocol::netty::handshaking::serverbound::Handshake0,
    version::ProtocolVersion,
};
use miners_level::chunk::ChunkColumn47;
use num_bigint::BigInt;
use rsa::pkcs8::Document;
use rsa::rand_core::OsRng;
use rsa::Pkcs1v15Encrypt;
use rsa::{pkcs8::EncodePublicKey, RsaPrivateKey};
use sha1::Digest;
use sha1::Sha1;
use uuid::Uuid;

const VERSION: i32 = 47;
const RSA_BIT_SIZE: usize = 2048;

#[async_std::main]
async fn main() {
    let mut args = std::env::args();
    let offline = args.any(|v| v == "--offline-mode");
    let compression = !args.any(|v| v == "--no-compression");

    let priv_key = Arc::new(RsaPrivateKey::new(&mut OsRng, RSA_BIT_SIZE).unwrap());
    let pub_key = Arc::new(priv_key.to_public_key().to_public_key_der().unwrap());

    let version = ProtocolVersion::new(VERSION).unwrap();

    let chunk = Arc::new(
        ChunkColumn47::from_nbt(
            &nbt::Nbt::decode(&mut Cursor::new(include_bytes!(
                "../level/test_data/testchunk.nbt"
            )))
            .unwrap(),
            true,
        )
        .unwrap(),
    );

    let listener = async_std::net::TcpListener::bind("127.0.0.1:25565")
        .await
        .unwrap();

    println!("Now listening on localhost:25565");
    loop {
        let (stream, _addr) = listener.accept().await.unwrap();
        let conn = miners::net::conn::Connection::new(stream.clone(), stream);
        spawn(accept(
            conn,
            version,
            priv_key.clone(),
            pub_key.clone(),
            chunk.clone(),
            offline,
            compression,
        ));
    }
}

async fn accept(
    mut conn: Conn,
    version: ProtocolVersion,
    priv_key: Arc<RsaPrivateKey>,
    pub_key: Arc<Document>,
    chunk: Arc<ChunkColumn47>,
    offline: bool,
    compression: bool,
) {
    let mut encoder = Encoder::new();
    match handshake(&mut conn, version).await.unwrap() {
        NextState0::Status => (),
        NextState0::Login => {
            let (read, write) = login(
                conn,
                &mut encoder,
                version,
                priv_key,
                pub_key,
                chunk,
                offline,
                compression,
            )
            .await
            .unwrap();
            play(read, write, version).await.unwrap()
        }
    };
}

async fn play(mut read: Reader, write: Writer, version: ProtocolVersion) -> anyhow::Result<()> {
    let keepalive_id = Arc::new(Mutex::new(rand::random::<i32>()));
    let write = Arc::new(Mutex::new(write));

    spawn(send_keepalive(write.clone(), keepalive_id.clone(), version));

    loop {
        match SbPlay::parse(read.read_encoded().await?.into_packet()?, version)? {
            SbPlay::KeepAlive7(packet) => {
                println!("received keepalive");
                let packet: KeepAlive7 = packet;
                let mut id = keepalive_id.lock().await;
                if packet.id != *id {
                    println!("keepalive id corrupted")
                }
                *id = rand::random();
            }
            _ => {}
        };
    }
}

async fn send_keepalive(
    write: Arc<Mutex<Writer>>,
    id: Arc<Mutex<i32>>,
    version: ProtocolVersion,
) -> anyhow::Result<()> {
    let mut encoder = Encoder::new();
    loop {
        sleep(Duration::from_secs(15)).await;
        let mut write = write.lock().await;
        write
            .write_packet(
                version,
                KeepAlive32 {
                    id: *id.lock().await,
                },
                &mut encoder,
            )
            .await?;
        write.flush().await?;
        println!("sent keepalive")
    }
}

async fn handshake(conn: &mut Conn, version: ProtocolVersion) -> anyhow::Result<NextState0> {
    match SbHandshaking::parse(conn.read_half.read_encoded().await?.into_packet()?, version)? {
        SbHandshaking::Handshake0(packet) => {
            // This is so rust-analyzer recognises the type.
            let packet: Handshake0 = packet;
            Ok(packet.next_state)
        }
    }
}

async fn login(
    mut conn: Conn,
    encoder: &mut Encoder,
    version: ProtocolVersion,
    priv_key: Arc<RsaPrivateKey>,
    pub_key: Arc<Document>,
    chunk: Arc<ChunkColumn47>,
    offline: bool,
    compression: bool,
) -> anyhow::Result<(Reader, Writer)> {
    let username = if let SbLogin::LoginStart0(packet) =
        SbLogin::parse(conn.read_half.read_encoded().await?.into_packet()?, version)?
    {
        let packet: LoginStart0 = packet;
        packet.username.to_string()
    } else {
        bail!("incorrect packet order")
    };

    let uuid = if !offline {
        // TODO: Fix client side decoding errors.
        let verify_token: [u8; 32] = rand::random();

        conn.write_half
            .write_packet(
                version,
                EncryptionRequest19 {
                    server_id: "".into(),
                    public_key: pub_key.as_bytes().into(),
                    verify_token: (&verify_token[..]).into(),
                },
                encoder,
            )
            .await?;

        conn.write_half.flush().await?;

        let secret = if let SbLogin::EncryptionResponse19(packet) =
            SbLogin::parse(conn.read_half.read_encoded().await?.into_packet()?, version)?
        {
            let packet: EncryptionResponse19 = packet;
            if &priv_key.decrypt(Pkcs1v15Encrypt, &packet.verify_token)? != &verify_token {
                bail!("verify token corrupted!")
            }
            priv_key.decrypt(Pkcs1v15Encrypt, &packet.secret)?
        } else {
            bail!("incorrect packet order")
        };

        let mut hash: Sha1 = Sha1::new();
        hash.update(b"");
        hash.update(&secret);
        hash.update(pub_key.as_bytes());
        let hash = BigInt::from_signed_bytes_be(hash.finalize().as_ref()).to_str_radix(16);
        let resp = isahc::get_async(format!(
            "https://sessionserver.mojang.com/session/minecraft/hasJoined?username={username}&serverId={hash}",
        ))
        .await?;
        if !(StatusCode::OK == resp.status()) {
            bail!(
                "request to sessionserver failed with status code: {}",
                resp.status()
            )
        }
        let mut body = String::new();
        resp.into_body().read_to_string(&mut body).await?;

        let json = serde_json::Value::from_str(&body)?;

        let uuid = Uuid::from_str(
            json.get("id")
                .map_or(Err(anyhow!("uuid not present in response")), |v| Ok(v))?
                .as_str()
                .map_or(Err(anyhow!("uuid is not string")), |v| Ok(v))?,
        )?;

        let test = json
            .get("id")
            .map_or(Err(anyhow!("uuid not present in response")), |v| Ok(v))?
            .as_str()
            .map_or(Err(anyhow!("uuid is not string")), |v| Ok(v))?;
        dbg!(test);

        conn.enable_encryption(secret.as_ref())?;
        StringUuid::from(uuid)
    } else {
        StringUuid::from(Uuid::from_bytes([
            0xa1, 0xa2, 0xa3, 0xa4, 0xb1, 0xb2, 0xa1, 0xa2, 0xa3, 0xa4, 0xb1, 0xb2, 0xa1, 0xa2,
            0xa3, 0xa4,
        ]))
    };

    if compression {
        conn.write_half
            .write_packet(version, SetCompression27 { threshold: 512 }, encoder)
            .await?;
        conn.write_half.flush().await?;
        conn.enable_compression(512);
    }

    conn.write_half
        .write_packet(
            version,
            Success0 {
                username: (&username).into(),
                uuid,
            },
            encoder,
        )
        .await
        .unwrap();

    conn.write_half.flush().await?;
    println!("{username} logged in!");
    println!("success0");

    let (read, mut write) = conn.split();

    write
        .write_packet(
            version,
            JoinGame29 {
                entity_id: 0,
                hardcore: false,
                gamemode: GameMode0::Creative,
                dimension: Dimension0::Overworld,
                difficulty: miners::protocol::netty::play::Difficulty0::Easy,
                max_players: 255,
                level_type: "default".into(),
                reduced_debug_info: false,
            },
            encoder,
        )
        .await?;
    write.flush().await?;

    println!("joingame29");
    std::thread::sleep(core::time::Duration::from_secs(2));

    write
        .write_packet(version, SpawnPosition6 { x: 0, z: 0, y: 60 }, encoder)
        .await?;
    write.flush().await?;

    write
        .write_packet(
            version,
            PlayerAbilities0 {
                invulnerable: true,
                flying: true,
                allow_flying: true,
                creative_mode: true,
                flying_speed: 0.05,
                fov: 0.1,
            },
            encoder,
        )
        .await?;

    write.flush().await?;

    write
        .write_packet(
            version,
            PositionAndLook6 {
                x: 0.0,
                y: 60.0,
                z: 0.0,
                yaw: 0.0,
                pitch: 0.0,
                relativity: miners::protocol::netty::play::clientbound::PositionAndLookBitfield6 {
                    x: false,
                    y: false,
                    z: false,
                    pitch: false,
                    yaw: false,
                },
            },
            encoder,
        )
        .await?;
    write.flush().await?;

    let mut chunk_data = Vec::<u8>::new();
    chunk.encode(&mut chunk_data)?;

    write
        .write_packet(
            version,
            ChunkData27 {
                chunk_x: 0,
                chunk_y: 0,
                continuous: true,
                primary_bitmap: chunk.primary_bitmap(),
                data: chunk_data.into(),
            },
            encoder,
        )
        .await
        .unwrap();
    write.flush().await?;

    dbg!("test");
    Ok((read, write))
}

type Conn = miners::net::conn::Connection<BufReader<TcpStream>, BufWriter<TcpStream>>;
type Reader = miners::net::conn::ReadHalf<BufReader<TcpStream>>;
type Writer = miners::net::conn::WriteHalf<BufWriter<TcpStream>>;
