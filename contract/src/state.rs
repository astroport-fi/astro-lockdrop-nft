use cw721_metadata_onchain::Metadata;
use cw_storage_plus::{Map, U8Key};

// Mapping of level to the level's metadata
pub const METADATA: Map<U8Key, Metadata> = Map::new("metadata");
