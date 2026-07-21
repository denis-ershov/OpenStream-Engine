//! DASH MPD: лёгкое XML-дерево + фильтры рекламных Period/AdaptationSet.

mod filter;
mod mpd;
mod xml;

pub use filter::{filter_ad_nodes, DashFilterRules, FilterStats};
pub use mpd::{AdaptationView, Mpd, PeriodView};
pub use xml::{parse_xml, serialize_xml, XmlElement, XmlNode, DashError};
