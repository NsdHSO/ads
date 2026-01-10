use anyhow::Result;
use clap::Parser;
use serde::Serialize;
use std::time::Duration;

#[derive(Debug, Parser)]
#[command(
    name = "publisher",
    about = "Publish telemetry JSON to Zenoh (drone/*)"
)]
struct Args {
    /// Key expression to publish
    #[arg(long, default_value = "drone/uav1/telemetry")]
    key: String,

    /// Provide raw JSON to publish. If omitted, fields below are used to build a Telemetry JSON.
    #[arg(long)]
    json: Option<String>,

    // Fields to synthesize a Telemetry JSON (if --json not provided)
    #[arg(long)]
    track: Option<u16>,
    #[arg(long)]
    lat: Option<f64>,
    #[arg(long)]
    lon: Option<f64>,
    #[arg(long)]
    alt_m: Option<i16>,
    #[arg(long)]
    speed_ms: Option<u16>,
    #[arg(long)]
    heading_deg: Option<f32>,

    /// Number of messages to publish
    #[arg(long, default_value_t = 1)]
    repeat: usize,
    /// Interval between messages (ms)
    #[arg(long, default_value_t = 1000)]
    interval_ms: u64,
}

#[derive(Debug, Serialize)]
struct TelemetryOut {
    track: u16,
    lat: f64,
    lon: f64,
    alt_m: i16,
    speed_ms: u16,
    heading_deg: f32,
}

fn synthesize(args: &Args) -> String {
    if let Some(j) = &args.json {
        return j.clone();
    }
    let t = TelemetryOut {
        track: args.track.unwrap_or(42),
        lat: args.lat.unwrap_or(45.1234567),
        lon: args.lon.unwrap_or(-122.9876543),
        alt_m: args.alt_m.unwrap_or(1500),
        speed_ms: args.speed_ms.unwrap_or(220),
        heading_deg: args.heading_deg.unwrap_or(271.5),
    };
    serde_json::to_string(&t).unwrap()
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    #[cfg(feature = "zenoh")]
    {
        let session = zenoh::open(zenoh::Config::default())
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        let publisher = session
            .declare_publisher(args.key.clone())
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        for i in 0..args.repeat {
            let payload = synthesize(&args);
            publisher
                .put(payload)
                .await
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            println!("published [{}/{}] to {}", i + 1, args.repeat, args.key);
            if i + 1 < args.repeat {
                tokio::time::sleep(Duration::from_millis(args.interval_ms)).await;
            }
        }
    }

    #[cfg(not(feature = "zenoh"))]
    {
        println!("publisher compiled without 'zenoh' feature. Rebuild with: cargo run -p publisher --features zenoh -- ...");
    }

    Ok(())
}
