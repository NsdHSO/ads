#[cfg(kani)]
mod proofs {
    use jseries::*;

    const LAT_MAX: u32 = (1 << 19) - 1; // 524_287
    const LON_MAX: u32 = (1 << 19) - 1; // 524_287
    const ALT_MAX: u16 = (1 << 14) - 1; // 16_383

    // Prove that from_geo never overflows and produces in-range packed fields
    // given physically valid inputs.
    #[kani::proof]
    fn from_geo_produces_in_range_fields() {
        let track: u16 = kani::any();
        let lat_deg: f64 = kani::any();
        let lon_deg: f64 = kani::any();
        let alt_m: f64 = kani::any();
        let speed_ms: u16 = kani::any();
        let heading_deg: u16 = kani::any();

        // Physical constraints: latitude ∈ [-90, 90], longitude ∈ [-180, 180], altitude ≥ 0 and bounded.
        kani::assume(lat_deg >= -90.0 && lat_deg <= 90.0);
        kani::assume(lon_deg >= -180.0 && lon_deg <= 180.0);
        // Keep altitude within the representable range to avoid u16 overflow on rounding.
        // ALT_MAX steps of 25 ft converted to meters.
        let alt_m_max = (ALT_MAX as f64) * 25.0 / 3.28084;
        kani::assume(alt_m >= 0.0 && alt_m <= alt_m_max);

        let v = J3_2AirTrack::from_geo(track, lat_deg, lon_deg, alt_m, speed_ms, heading_deg);
        println!("{:?}", v);
        // Ranges implied by the bit-widths
        assert!(v.latitude <= LAT_MAX);
        assert!(v.longitude <= LON_MAX);
        assert!(v.altitude <= ALT_MAX);

        // Derived field is masked to 12 bits.
        assert_eq!(v.track_number, track & 0x0FFF);

        // Other fields are just copies
        assert_eq!(v.track, track);
        assert_eq!(v.speed_ms, speed_ms);
        assert_eq!(v.heading_cdeg, heading_deg);
    }
}
