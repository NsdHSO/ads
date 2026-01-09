//! Link 16 J-Series parsing/serialization (prototype)
//! This is a simplified, non-authoritative representation suitable for scaffolding.

use core::fmt;
use deku::prelude::*;

pub const MSG_ID_J3_2: u8 = 0x32; // Prototype identifier for J3.2 Air Track

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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

impl JMessage {
    pub fn from_bytes(input: &[u8]) -> Result<Self, Error> {
        if input.len() < 1 {
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
    /// 16-bit track number
    #[deku(bytes = 2)]
    pub track: u16,
    /// latitude scaled by 1e7 (degrees * 1e7)
    #[deku(bytes = 4)]
    pub lat_e7: i32,
    /// longitude scaled by 1e7 (degrees * 1e7)
    #[deku(bytes = 4)]
    pub lon_e7: i32,
    /// altitude in meters
    #[deku(bytes = 2)]
    pub alt_m: i16,
    /// speed in m/s
    #[deku(bytes = 2)]
    pub speed_ms: u16,
    /// heading in degrees * 100 (0..=35999)
    #[deku(bytes = 2)]
    pub heading_cdeg: u16,
}

impl J3_2AirTrack {
    pub fn from_geo(
        track: u16,
        lat_deg: f64,
        lon_deg: f64,
        alt_m: i16,
        speed_ms: u16,
        heading_deg: f32,
    ) -> Self {
        let lat_e7 = (lat_deg * 10_000_000.0).round() as i32;
        let lon_e7 = (lon_deg * 10_000_000.0).round() as i32;
        let heading_cdeg = ((heading_deg.rem_euclid(360.0)) * 100.0).round() as u16;
        Self {
            track,
            lat_e7,
            lon_e7,
            alt_m,
            speed_ms,
            heading_cdeg,
        }
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
            1500,
            220,
            271.5,
        ));
        let bytes = msg.to_bytes().unwrap();
        let parsed = JMessage::from_bytes(&bytes).unwrap();
        assert_eq!(msg, parsed);
    }
}

// Kani proof harness (compiled only under the Kani verifier)
#[cfg(kani)]
mod proofs {
    use super::*;

    #[kani::proof]
    fn no_panic_on_valid_j3_2() {
        // Create an arbitrary J3.2 body and ensure (de)serialization roundtrips.
        let track: u16 = kani::any();
        let lat: i32 = kani::any();
        let lon: i32 = kani::any();
        let alt: i16 = kani::any();
        let spd: u16 = kani::any();
        let hdg: u16 = kani::any();
        let body = J3_2AirTrack {
            track,
            lat_e7: lat,
            lon_e7: lon,
            alt_m: alt,
            speed_ms: spd,
            heading_cdeg: hdg,
        };
        let msg = JMessage::J3_2(body.clone());
        let bytes = msg.to_bytes().unwrap();
        let parsed = JMessage::from_bytes(&bytes).unwrap();
        match parsed {
            JMessage::J3_2(b) => assert!(b == body),
        }
    }
}
