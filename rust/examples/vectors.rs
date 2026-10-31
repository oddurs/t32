use t32::*;
fn civil(fx: u128) -> String {
    let u = fx_to_unix_ns(fx);
    let s = u.div_euclid(1_000_000_000) as i64; let ns = u.rem_euclid(1_000_000_000);
    let (y, m, d) = civil_from_days(s.div_euclid(86400));
    let r = s.rem_euclid(86400);
    format!("{y:05}-{m:02}-{d:02}T{:02}:{:02}:{:02}.{:09}Z", r/3600, r%3600/60, r%60, ns)
}
fn main() {
    let cases: &[(&str, (i64,u32,u32,u32,u32,u32,u32), u8)] = &[
        ("T32 epoch", (1999,12,31,23,59,28,0), 8),
        ("Unix epoch", (1970,1,1,0,0,0,0), 8),
        ("UTC goes integer", (1972,1,1,0,0,0,0), 8),
        ("Apollo 11 landing", (1969,7,20,20,17,40,0), 8),
        ("Last leap second, :59", (2016,12,31,23,59,59,0), 8),
        ("Last leap second, :60", (2016,12,31,23,59,60,0), 8),
        ("Next second", (2017,1,1,0,0,0,0), 8),
        ("Spec date", (2026,10,31,3,13,0,0), 8),
        ("Spec date + 0.5 s", (2026,10,31,3,13,0,500_000_000), 10),
        ("Spec date, ms", (2026,10,31,3,13,0,123_000_000), 12),
        ("Royal Observatory stone (Greg. 1675-08-20)", (1675,8,20,0,0,0,0), 8),
    ];
    for (name, (y,mo,d,h,mi,s,ns), n) in cases {
        let fx = utc_to_fx(*y,*mo,*d,*h,*mi,*s,*ns).unwrap();
        let st = Stamp::from_biased_fx(fx, *n).unwrap();
        println!("| {name} | {y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}{} UTC | `{st}` |", if *ns>0 {format!(".{:09}", ns).trim_end_matches('0').to_string()} else {String::new()});
    }
    for s in ["00000000","ZZZZZZZZ","G","G0S7","G0S7G6"] {
        let st = Stamp::parse(s).unwrap();
        println!("{s}: start {} end {}", civil(st.start_fx()), civil(st.end_fx()));
    }
}
