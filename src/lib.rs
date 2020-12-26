use std::str::FromStr;
use std::fmt;
use termion::color;

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
    
    fn from_str(s: &str) -> Result<FlightCategory, ()> {
        match s {
            "LIFR" => Ok(FlightCategory::LIFR),
            "IFR" => Ok(FlightCategory::IFR),
            "MVFR" => Ok(FlightCategory::MVFR),
            "VFR" => Ok(FlightCategory::VFR),
            _ => Ok(FlightCategory::UNKN),
        }
    }
}

macro_rules! color_write {
    ($out:ident,$s:expr,$c:expr) => {
        write!($out, "{}{}{}", color::Fg($c), $s, color::Fg(color::Reset))
    }
}

impl fmt::Display for FlightCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FlightCategory::LIFR => color_write!(f, "LIFR", color::Magenta),
            FlightCategory::IFR => color_write!(f, "IFR", color::Red),
            FlightCategory::MVFR => color_write!(f, "MVFR", color::Blue),
            FlightCategory::VFR => color_write!(f, "VFR", color::Green),
            FlightCategory::UNKN => write!(f, "UNKN"),
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

pub fn extract_metars(metar_xml: &str) -> Result<impl Iterator<Item = Metar>, roxmltree::Error> {
    let doc = roxmltree::Document::parse(metar_xml)?;
    let nodes = doc.descendants()
        .filter(|n| n.has_tag_name("METAR"))
        .map(|n| n.descendants())
        .map(|mut descendant| Metar{
            raw_text: descendant.clone().find(|n| n.has_tag_name("raw_text")).unwrap().text().unwrap().to_string(),
            flight_category: get_flight_category_if_node_exists(descendant.find(|n| n.has_tag_name("flight_category"))),
        })
        .collect::<Vec<_>>()
        .into_iter()
        .rev();
    Ok(nodes)
}

fn get_flight_category_if_node_exists(res: Option<roxmltree::Node>) -> FlightCategory {
    if let Some(node) = res {
        if let Some(text) = node.text() {
            return FlightCategory::from_str(text).unwrap()
        }
    }
    FlightCategory::UNKN
}

pub fn get_site_metars(site: &str) -> Result<String, reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    let res = client.get("https://www.aviationweather.gov/adds/dataserver_current/httpparam")
        .query(&[
            ("dataSource", "metars"),
            ("requestType", "retrieve"),
            ("format", "xml"),
            ("stationString", site),
            ("hoursBeforeNow", "1")
        ]);
    
    Ok(res.send()?.text()?)
}