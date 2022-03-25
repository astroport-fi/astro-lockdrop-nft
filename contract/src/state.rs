use cw721_metadata_onchain::Metadata;
use cw_storage_plus::{Item, Map, U8Key};
use cosmwasm_std::Addr;

// Address of the minter
pub const MINTER: Item<Addr> = Item::new("minter");

// Mapping of level to the level's metadata
pub const METADATA: Map<U8Key, Metadata> = Map::new("metadata");
