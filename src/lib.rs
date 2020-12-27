use std::str::FromStr;
use std::fmt;
use termion::{color, style};

#[derive(Debug)]
pub enum Error {
    NoRawText,
    XMLParseError,
}

#[derive(Debug)]
pub enum FlightCategory {
    LIFR,
    IFR,
    MVFR,
    VFR,
    UNKN,
}

impl FromStr for FlightCategory {
    type Err = ();
    
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "LIFR" => Ok(Self::LIFR),
            "IFR" => Ok(Self::IFR),
            "MVFR" => Ok(Self::MVFR),
            "VFR" => Ok(Self::VFR),
            _ => Ok(Self::UNKN),
        }
    }
}

macro_rules! color_write {
    ($out:ident,$s:expr,$c:expr) => {
        write!($out, "{}{}{}{}{}",
            style::Bold,
            color::Fg($c),
            $s,
            color::Fg(color::Reset),
            style::Reset)
    }
}

impl fmt::Display for FlightCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LIFR => color_write!(f, "LIFR", color::Magenta),
            Self::IFR => color_write!(f, "IFR", color::Red),
            Self::MVFR => color_write!(f, "MVFR", color::Blue),
            Self::VFR => color_write!(f, "VFR", color::Green),
            Self::UNKN => write!(f, "UNKN"),
        }
    }
}

pub struct Metar {
    raw_text: String,
    flight_category: FlightCategory
}

impl fmt::Display for Metar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.flight_category, self.raw_text)
    }
}

pub fn extract_metars(metar_xml: &str) -> Result<impl Iterator<Item = Metar>, Error> {
    let doc = match roxmltree::Document::parse(metar_xml) {
        Ok(res) => res,
        Err(_) => return Err(Error::XMLParseError),
    };

    let nodes = doc.descendants()
        .filter(|n| n.has_tag_name("METAR"))
        .map(|n| n.descendants())
        .map(|mut descendant| Metar {
            raw_text: descendant.clone()
                .find(|n| n.has_tag_name("raw_text"))
                .and_then(|n| n.text())
                .unwrap_or_default()
                .to_string(),
            flight_category: {
                let flight_category_node = descendant.find(|n| n.has_tag_name("flight_category"));
                get_flight_category_if_node_exists(flight_category_node)
            },
        })
        .collect::<Vec<_>>()
        .into_iter()
        .rev();
    Ok(nodes)
}

fn get_flight_category_if_node_exists(res: Option<roxmltree::Node>) -> FlightCategory {
    res
        .and_then(|node| node.text())
        .and_then(|text| FlightCategory::from_str(text).ok())
        .unwrap_or(FlightCategory::UNKN)
}

pub fn get_site_metars(sites: Vec<String>) -> Result<String, reqwest::Error> {
    let site_string = sites.join(",");
    let client = reqwest::blocking::Client::new();
    let res = client.get("https://www.aviationweather.gov/adds/dataserver_current/httpparam")
        .query(&[
            ("dataSource", "metars"),
            ("requestType", "retrieve"),
            ("format", "xml"),
            ("stationString", &site_string),
            ("hoursBeforeNow", "1")
        ]);
    
    Ok(res.send()?.text()?)
}