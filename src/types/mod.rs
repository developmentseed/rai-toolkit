pub mod net;
pub mod context;
pub mod name;
pub mod polygon;
pub use net::Network;

pub trait AsTSV {
    fn as_tsv(self) -> String;
}

pub trait AsFeat {
    fn as_feat(self) -> geojson::Feature;
}
