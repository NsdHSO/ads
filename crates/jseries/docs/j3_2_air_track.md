# J3.2 “Air Track” — Developer Notes (Unclassified)

Purpose
J3.2 is a Link 16 J‑series surveillance message used to create/update airborne tracks on the Recognized Air Picture (RAP). This document tells you what you can implement in an open repo today, what must live behind a licensed/spec‑gated plug‑in, and how to route J3.2 toward real platforms (e.g., via a Link 16 gateway for delivery to an F‑22) or to lab simulators.

Compliance boundary (read first)
- Do not reproduce restricted STANAG 5516/MIL‑STD‑6016 bit layouts in public code.
- This repo provides: (a) a neutral AirTrack schema; (b) word/framing/parity utilities; (c) a SpecPack trait; (d) open SISO Link‑16 simulation mappings for tests.
- Integrators with licensed access implement the private J3.2 packer that maps fields into the actual bit layout.

High‑level semantics (safe to document)
- J3.2 conveys: track identity/classification; position (lat/lon/alt); kinematics (speed/course, optional climb); track quality/strength; timing/source meta.
- Fixed‑format J‑series messages occupy 3 data words. Each Link 16 word is 75 bits: 70 information bits + 4 parity bits + 1 spare bit (bit 70).
- The terminal/radio handles waveform coding (e.g., Reed–Solomon, interleaving, frequency hopping) and on‑air crypto. Your software must only produce valid 75‑bit words and correct per‑word parity.

Neutral data model (for your APIs and Zenoh topics)
```jsonc
{
  "track_id": 123456,           // integer, unique in your system
  "time_ms": 1737086400123,     // epoch ms
  "lat_e7": 452345678,          // degrees * 1e7 (int, signed)
  "lon_e7": 26345678,           // degrees * 1e7 (int, signed)
  "alt_dm": 10230,              // decimeters (int, signed)
  "spd_cmps": 23045,            // cm/s (int)
  "crs_cdeg": 12345,            // centi-degrees 0..35999
  "climb_cmps": -120,           // cm/s (int, signed)
  "identity_code": 3,           // enum (pending/unknown/friend/neutral/suspect/hostile)
  "q_track": 4,                  // quality bucket 0..7 (example)
  "src": 512                    // source/unit code per local policy
}
```

Bit/word responsibilities (what this repo will implement)
- JWord (75‑bit) with fields: payload70 (u128, top‑70 bits used), parity4 (u8), spare (u1).
- Fixed‑format frame helper for 3 words with a pluggable ParityProvider:
  - Inputs: header bits 4..=18 (as required by your licensed spec) and the three 70‑bit payloads.
  - Output: 3 words with parity4 set per word; spare set to 0 unless directed otherwise.
- SpecPack trait: neutral AirTrack → [payload70; 3]. The private, licensed crate implements the actual J3.2 field packing.

Deterministic scaling rules
- Use integer‑only scaling: lat/lon in 1e‑7 deg; altitude in decimeters; speed in cm/s; course in centi‑degrees.
- Clamp out‑of‑range values before packing; surface these clamps as counters and logs.

Verification hooks
- Kani proofs: no panics; slice/bit‑index safety; round‑trip payload70 to bytes and back; packing monotonicity (within declared ranges).
- Optional: Verus specs for pack→unpack==id (SISO mapping) and bounds preservation.

How to send J3.2 toward a real aircraft (e.g., an F‑22)
Important: You do not “IP send” directly to an aircraft. Delivery occurs over a tactical data link network using certified terminals, COMSEC, and timing. Typical paths:
1) Via a Link 16 Gateway (JREAP‑C over IP)
   - Your gateway encodes J3.2 into valid 75‑bit words and publishes them over JREAP‑C to a Tactical Data Link (TDL) gateway.
   - The TDL gateway injects the message onto the Link 16 network via a certified terminal (e.g., MIDS family) with crypto and time sync.
   - The aircraft (including F‑22, subject to its configuration) receives the J‑series traffic as a standard Link 16 participant.
2) Via a locally connected Link 16 terminal
   - Your host connects to a terminal over its vendor ICD (Ethernet/serial). You provide 75‑bit words/J‑message PDUs as that ICD requires; the terminal handles RF/crypto.
   - This mode requires accredited hardware, keys (COMSEC), and operational authorization.
3) Lab/simulation
   - Use SISO Link‑16 DIS/HLA simulation bindings. This path is fully open and is recommended for CI, demos, and integration tests.

ASCII flows
```
[Zenoh JSON AirTrack] -> [SpecPack (private) + jseries_core] -> [J3.2 words]
   -> (A) JREAP‑C/TLS -> [TDL Gateway] -> [MIDS Terminal] -> RF -> [Aircraft]
   -> (B) Vendor ICD -> [Local Terminal] -> RF -> [Aircraft]
   -> (C) DIS/HLA -> [Sim Range / Test Toolchain]
```

JREAP‑C notes (for your implementation boundary)
- JREAP‑C (MIL‑STD‑3011) is the doctrinal way to carry J‑series over TCP/UDP/IP between sites.
- In this repo, we terminate a TLS/mTLS session (rustls) and encapsulate the J‑series payload per the gateway’s ICD. Keep this module configurable and pluggable.
- Use allow‑/deny‑lists and per‑topic rate limits to avoid oversubscription on narrow tactical links.

Security
- Transport: rustls (TLS 1.3) with mTLS, cert pinning, session ticket rotation.
- PQC: enable hybrid KX groups (e.g., X25519+ML‑KEM‑768) when both peers support them; fall back cleanly.
- Supply SBOM/provenance; run cargo‑audit; deny(warnings); clippy pedantic for security‑critical crates.

Operational cautions (non‑technical)
- Time and slotting are critical on Link 16. Your software must not attempt to emulate network time/slot assignment; that is terminal/gateway function.
- Do not attempt on‑air transmission without authorized equipment, keying material, and operational approval.

Interfaces you can expose now
- CLI flags or JSON5 config for:
  - zenoh endpoints; topic allow/deny; max frequency per topic
  - mapping rules: <zenoh key> ⇄ <SpecPack impl + message type>
  - JREAP‑C peer(s) and TLS settings (certs, ciphers, PQ groups)
  - simulation mode: DIS/HLA publish/subscribe options

Minimal acceptance tests (open)
- Encode/decode SISO mapping round‑trip
- Word parity non‑zero and stable for unchanged inputs
- Throughput: ≤ 1 ms/encode on x86‑64 release
- Kani proofs pass on CI

References (open, for design justification only)
- Federated Mission Networking (FMN) service profiles referencing RAP and J‑series use
- SISO Link‑16 Simulation (DIS/HLA) for lab representations
- MIL‑STD‑3011 (JREAP) for IP carriage of J‑series
- Rustls and aws‑lc‑rs hybrid KX support (TLS 1.3)

Next steps (actionable)
- Implement jseries_core::JWord and fixed‑format frame with ParityProvider trait.
- Add SpecPack trait and SISO demo packer.
- Provide a JREAP‑C client stub behind a feature flag.
- Wire Kani proofs and basic benches.
