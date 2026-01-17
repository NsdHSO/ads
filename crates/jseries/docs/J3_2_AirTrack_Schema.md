# J3.2 Air Track — Schema and Visual Cheatsheet (Unclassified)

Scope
This page gives you a picture-first guide to implementing and routing J3.2 “Air Track” without reproducing restricted STANAG 5516/MIL‑STD‑6016 bit layouts. It defines: a neutral AirTrack schema, packing flow into 75‑bit words, and delivery paths (gateway/terminal/sim).

Guardrails
- Do not publish restricted bit positions/widths. Keep the actual J3.2 mapping in a private, licensed crate.
- This repo may implement word structure (75 bits = 70 info + 4 parity + 1 spare) and a pluggable parity provider.

At a glance (flow)
```
Zenoh JSON AirTrack
    │
    ▼
[SpecPack (private, licensed)]   ─────────►  [payload70[0], payload70[1], payload70[2]]
    │                                            (3 × 70‑bit data words)
    ▼
[Frame builder + ParityProvider] ─────────►  [JWord, JWord, JWord]  (3 × 75‑bit words)
    │
    ├─ (A) JREAP‑C/TLS  ─►  TDL Gateway ─► Link 16 Terminal ─► RF ─► Aircraft
    ├─ (B) Vendor ICD   ─►  Link 16 Terminal ─► RF ─► Aircraft
    └─ (C) DIS/HLA Sim  ─►  Test/Range tools
```

Neutral AirTrack schema (for APIs and topics)
```jsonc
{
  "track_id": 123456,
  "time_ms": 1737086400123,
  "lat_e7": 452345678,   // deg × 1e7 (int)
  "lon_e7": 26345678,    // deg × 1e7 (int)
  "alt_dm": 10230,       // decimeters
  "spd_cmps": 23045,     // cm/s
  "crs_cdeg": 12345,     // centi‑degrees 0..35999
  "climb_cmps": -120,    // cm/s (signed)
  "identity_code": 3,    // enum: pending/unknown/friend/neutral/suspect/hostile
  "q_track": 4,          // quality bucket (example 0..7)
  "src": 512             // source/unit policy field
}
```

Quantization rules (deterministic, integer‑only)
- lat_e7 = round(latitude_deg × 10^7) in i32
- lon_e7 = round(longitude_deg × 10^7) in i32
- alt_dm in i32; spd_cmps in u32; crs_cdeg in u16 (wrap 0..=35999); climb_cmps in i16
- Clamp to declared ranges before packing. Emit counters for clamps.

Word and message visuals (conceptual)
```
75‑bit word (MSB→LSB):
┌─────────────────────────────────────────── 70 info bits ───────────────────────────────────────────┬──── parity4 ────┬─ spare ─┐
│                                        payload70[69:0]                                           │  p[3:0]         │   s     │
└───────────────────────────────────────────────────────────────────────────────────────────────────┴─────────────────┴─────────┘

Fixed‑format J‑series message (3 data words):
┌──────────────┬──────────────┬──────────────┐
│  JWord #1    │  JWord #2    │  JWord #3    │   (+ header/context outside payload70 via SpecPack)
└──────────────┴──────────────┴──────────────┘
```

Interfaces (public, safe)
- SpecPack: neutral AirTrack → [u128; 3] (three 70‑bit payloads; MSB‑first usage)
- ParityProvider: computes 4 parity bits per word given header context and the 3×70 info bits
- JWord: { payload70: u128, parity4: u8, spare: u1 }

Example pack/assemble (pseudocode)
```text
payloads = SpecPack::pack_air_track_fixed(&air_track)  // [u128; 3], each < 2^70
frame    = JFixedFrame::new(header_bits_4_18, payloads)
frame_p  = frame.with_parity(&MyParityProvider)
words    = frame_p.words  // [JWord; 3]
```

Where the private mapping lives
- Private crate (e.g., jseries_spec_stanag5516) implements SpecPack for J3.2: field widths, ordering, conditional bits, scaling per the licensed spec. Keep this out of the public repo.

Sending to a real aircraft (e.g., F‑22)
- Preferred: JREAP‑C over IP to a Tactical Data Link gateway. Your app emits J‑series PDUs/words over a secure session (TLS/mTLS). The gateway/terminal handles crypto, timing, and RF. You cannot “send Link 16 over raw IP” to an aircraft.
- Alternate: Direct vendor ICD to a locally attached terminal.
- Labs: DIS/HLA Link‑16 Simulation bindings for full‑open testing without restricted content.

Security and transport
- Use rustls (TLS 1.3) with mTLS and cert pinning. Prefer hybrid PQ groups (e.g., X25519+ML‑KEM‑768) when both peers support them; fall back gracefully.
- Rate‑limit per topic and provide allow/deny lists for narrow tactical links.

Verification
- Kani proofs for: no panics, slice bounds, payload70 round‑trip, monotonic scaling.
- Optional Verus specs: pack→unpack id for SISO mapping; bounds preservation.

Minimal acceptance (open tests)
- SISO mapping encode/decode round‑trip OK
- Parity bits stable for identical inputs; spare == 0 unless configured otherwise
- ≤ 1 ms per encode on x86‑64 release build

Notes
- Do not attempt waveform functions (time slotting, interleaving, on‑air crypto); terminals/gateways own those responsibilities.
- Keep field enums (identity, activity, type) abstract in public code; map to exact values only inside licensed modules.
