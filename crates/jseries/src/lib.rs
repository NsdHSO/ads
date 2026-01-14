//! Link 16 J-Series parsing/serialization (prototype)
//! This is a simplified, non-authoritative representation suitable for scaffolding.

use core::fmt;
use deku::prelude::*;
use std::fmt::Formatter;

pub const MSG_ID_J3_2: u8 = 0x32; // Prototype identifier for J3.2 Air Track
const LAT_SCALE: f64 = 524287.0 / 180.0; // 19-bit mapping for -90 to +90
const LON_SCALE: f64 = 524287.0 / 360.0; // 19-bit mapping for -180 to +180
const ALT_STEP: f64 = 25.0; // Standard 25ft altitude increments
#[derive(Debug, Clone)]
pub enum Error {
    Unsupported(u8),
    Short(usize),
    Deku(String),
}

impl From<deku::error::DekuError> for Error {
    fn from(e: deku::error::DekuError) -> Self {
        Self::Deku(e.to_string())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::Unsupported(k) => write!(f, "unsupported message kind: {k:02x}"),
            Error::Short(n) => write!(f, "buffer too short: {n} bytes"),
            Error::Deku(s) => write!(f, "deku error: {s}"),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JMessage {
    J3_2(J3_2AirTrack),
}

impl fmt::Display for JMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            JMessage::J3_2(track) => write!(f, "J3.2 Message: {}", track),
        }
    }
}

impl JMessage {
    pub fn from_bytes(input: &[u8]) -> Result<Self, Error> {
        if input.is_empty() {
            return Err(Error::Short(input.len()));
        }
        let kind = input[0];
        match kind {
            MSG_ID_J3_2 => {
                // remaining is the body
                let (_, body) = J3_2AirTrack::from_bytes((&input[1..], 0))?;
                Ok(JMessage::J3_2(body))
            }
            other => Err(Error::Unsupported(other)),
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        match self {
            JMessage::J3_2(v) => {
                let mut out = Vec::with_capacity(1 + 16);
                out.push(MSG_ID_J3_2);
                let body = v.to_bytes()?;
                out.extend(body);
                Ok(out)
            }
        }
    }
}

/// Prototype J3.2 Air Track body (highly simplified)
/// Big-endian, fixed-width layout to keep bit/byte packing explicit.
#[derive(Debug, Clone, PartialEq, Eq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct J3_2AirTrack {
    #[deku(bytes = 2)]
    pub track: u16,
    #[deku(bits = 19)]
    pub latitude: u32,
    #[deku(bits = 19)]
    pub longitude: u32,
    #[deku(bits = 12)]
    pub track_number: u16,
    #[deku(bits = 14)]
    pub altitude: u16,
    #[deku(bits = 5)]
    pub parity: u8,
    #[deku(bytes = 2)]
    pub speed_ms: u16,
    #[deku(bytes = 2)]
    pub heading_cdeg: u16,
}

impl J3_2AirTrack {
    pub fn from_geo(
        track: u16,
        lat_deg: f64,
        lon_deg: f64,
        alt_meters: f64,
        speed_ms: u16,
        heading_deg: u16,
    ) -> Self {
        Self {
            track,
            track_number: track & 0x0FFF,
            latitude: ((lat_deg + 90.0) * LAT_SCALE).round() as u32, // 19-bit squish
            longitude: ((lon_deg + 180.0) * LON_SCALE).round() as u32, // 19-bit squish
            altitude: (alt_meters * 3.28084 / ALT_STEP).round() as u16, // 14-bit squish
            parity: 0,
            speed_ms,
            heading_cdeg: heading_deg,
        }
    }
}

impl fmt::Display for J3_2AirTrack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Track #{}: [Lat: {}, Lon: {}] Speed: {}m/s, Alt: {}ft",
            self.track_number,
            self.latitude,
            self.longitude,
            self.speed_ms,
            self.altitude
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_j3_2() {
        let msg = JMessage::J3_2(J3_2AirTrack::from_geo(
            42,
            45.1234567,
            -122.9876543,
            1500.9,
            220,
            271,
        ));
        let bytes = msg.to_bytes().unwrap();
        let parsed = JMessage::from_bytes(&bytes).unwrap();
        assert_eq!(msg, parsed);
    }
}
