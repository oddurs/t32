//! A small T32 converter.
//!
//! ```text
//! cargo run --example t32                              # now, with every prefix
//! cargo run --example t32 -- 2026-10-31T03:13:00.500Z  # UTC to stamps
//! cargo run --example t32 -- G0S7                      # a stamp's interval
//! ```

use t32::{Duration, Error, Instant, Stamp, UtcDateTime};

fn main() {
    let arg = std::env::args().nth(1);
    if let Err(e) = run(arg.as_deref()) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run(arg: Option<&str>) -> Result<(), Error> {
    match arg {
        // A date-time has a colon; a stamp never does.
        Some(s) if s.contains(':') => {
            let utc: UtcDateTime = s.parse()?;
            let instant = Instant::from_utc(utc)?;
            // Never claim more resolution than the input has.
            let digits = s.rsplit_once('.').map_or(0, |(_, f)| f.len() - 1) as u32;
            let resolution = Duration::from_nanos(10i128.pow(9 - digits.min(9)))?;
            let len = Stamp::max_len(resolution);
            print_ladder(instant, len)
        }
        Some(s) => {
            let stamp: Stamp = s.parse()?;
            println!("stamp   {stamp}  ({})", stamp.display_form());
            println!("start   {}", stamp.start().to_utc());
            match stamp.end() {
                Some(end) => println!("end     {}", end.to_utc()),
                None => println!("end     (the end of the representable range)"),
            }
            println!("middle  {}", stamp.midpoint().to_utc());
            let width = stamp.width();
            println!("width   {width} (T32 duration) = {} ns", width.as_nanos());
            Ok(())
        }
        None => print_ladder(Instant::now()?, 10),
    }
}

fn print_ladder(instant: Instant, len: u8) -> Result<(), Error> {
    println!("{}", instant.to_utc());
    for n in 1..=len {
        let stamp = Stamp::new(instant, n)?;
        let end = stamp
            .end()
            .map_or_else(|| "end of range".into(), |e| e.to_utc().to_string());
        println!(
            "{:>2}  {:<16}  {} .. {}",
            n,
            stamp.to_string(),
            stamp.start().to_utc(),
            end
        );
    }
    Ok(())
}
