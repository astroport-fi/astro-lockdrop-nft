use cosmwasm_std::{Binary, Empty};
use cw721::{Expiration, ContractInfoResponse};
use cw721_metadata_onchain::Metadata;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct InstantiateMsg {
    /// Name and symbol of the NFT series
    pub contract_info: ContractInfoResponse,
    /// Account who can mint NFTs
    pub minter: String,
    /// Metadata for each level. The 1st element in the array is for Lv. 1, the 2nd is for Lv. 2, etc.
    pub metadatas: Vec<Metadata>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    //----------------------------------------
    // Commands specific to this NFT
    //----------------------------------------

    /// Mint a single item to a owner
    Mint {
        level: u8,
        owners: Vec<String>,
    },

    //----------------------------------------
    // Generic CW721 commands
    //----------------------------------------

    TransferNft {
        recipient: String,
        token_id: String,
    },
    SendNft {
        contract: String,
        token_id: String,
        msg: Binary,
    },
    Approve {
        spender: String,
        token_id: String,
        expires: Option<Expiration>,
    },
    Revoke {
        spender: String,
        token_id: String,
    },
    ApproveAll {
        operator: String,
        expires: Option<Expiration>,
    },
    RevokeAll {
        operator: String,
    },
}

/// We use the generic CW721 query messages
pub type QueryMsg = cw721::Cw721QueryMsg;

/// We don't need any input parameter for migration
pub type MigrateMsg = Empty;
