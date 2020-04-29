pub mod net;
pub use net::Network;

pub trait AsTSV {
    fn as_tsv(self) -> String;
}

pub trait AsFeat {
    fn as_feat(self) -> geojson::Feature;
}
