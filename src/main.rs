use std::env;
use metars::{get_site_metars, extract_metars};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: metars <icao>");
        return;
    }
    let sites = &args[1..];
    let body = get_site_metars(&sites[0]).unwrap();
    let metars = extract_metars(&body).unwrap();
    for metar in metars {
        println!("{}", metar);
    }
}