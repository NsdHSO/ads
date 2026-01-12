use anyhow::Result;
use clap::Parser;
use jseries::{J3_2AirTrack, JMessage};
use std::net::SocketAddr;

#[derive(Debug, Parser)]
#[command(name = "bridge", about = "ADS Secure Translator bridge (prototype)")]
struct Args {
    /// Zenoh selector to subscribe (e.g., drone/**)
    #[arg(long, default_value = "drone/**")]
    subscribe: String,
    /// UDP sink address for Link 16 bytes (e.g., 127.0.0.1:5000)
    #[arg(long, default_value = "127.0.0.1:5000")]
    sink: SocketAddr,
    /// Use E2EE with PSK hex (optional)
    #[arg(long)]
    psk_hex: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct Telemetry {
    track: u16,
    lat: f64,
    lon: f64,
    alt_m: f64,
    speed_ms: u16,
    heading_deg: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let sess = args.psk_hex.as_deref().map(hex_to_session);
    let sock = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;

    #[cfg(feature = "zenoh")]
    {
        // Zenoh 1.x API: open() and declare_subscriber() are async and return Results directly.
        let config = zenoh::config::Config::default();
        let session = zenoh::open(config)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        let sub = session
            .declare_subscriber(args.subscribe.clone())
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        println!(
            "bridge: listening on Zenoh selector '{}' -> UDP {}",
            args.subscribe, args.sink
        );
        loop {
            let sample = sub
                .recv_async()
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            // Extract payload text from ZBytes
            if let Ok(text) = sample.payload().try_to_string() {
                if let Ok(t) = serde_json::from_str::<Telemetry>(&text) {
                    let j = JMessage::J3_2(J3_2AirTrack::from_geo(
                        t.track,
                        t.lat,
                        t.lon,
                        t.alt_m,
                        t.speed_ms,
                        t.heading_deg,
                    ));
                    let mut bytes = j.to_bytes()?;
                    if let Some(s) = &sess {
                        bytes = s.seal(b"j3.2", &bytes)?;
                    }
                    sock.send_to(&bytes, args.sink).await?;
                }
            }
        }
    }

    #[cfg(not(feature = "zenoh"))]
    {
        println!("bridge compiled without 'zenoh' feature. Rebuild with: cargo run -p bridge --features zenoh -- ...");
        Ok(())
    }
}

fn hex_to_session(hex: &str) -> e2ee::Session {
    let data = hex::decode(hex).expect("invalid hex");
    e2ee::session_from_psk(&data)
}
