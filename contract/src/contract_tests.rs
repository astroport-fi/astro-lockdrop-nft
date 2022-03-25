use cosmwasm_std::testing::{
    mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_std::{from_binary, Deps, OwnedDeps};
use cw721::{
    AllNftInfoResponse, ContractInfoResponse, NftInfoResponse, NumTokensResponse, OwnerOfResponse,
    TokensResponse,
};
use cw721_base::ContractError;
use cw721_metadata_onchain::{Metadata, Trait};
use serde::de::DeserializeOwned;

use crate::contract::{execute, instantiate, query};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

fn create_init_msg() -> InstantiateMsg {
    let contract_info = ContractInfoResponse {
        name: String::from("Astroport Lockdrop NFT"),
        symbol: String::from("n/a"),
    };

    let metadata_lv_1 = Metadata {
        name: Some(String::from("Astroport Lockdrop NFT - Lv. 1")),
        description: Some(String::from("You have participated in any phase of the Astroport lockdrop")),
        external_url: Some(String::from("https://astroport.fi")),
        image: Some(String::from("ipfs://hash_1")),
        attributes: Some(vec![
            Trait {
                display_type: None,
                trait_type: String::from("protocol"),
                value: String::from("astroport"),
            },
            Trait {
                display_type: None,
                trait_type: String::from("level"),
                value: String::from("1"),
            },
        ]),
        image_data: None,
        background_color: None,
        animation_url: None,
        youtube_url: None,
    };

    let metadata_lv_2 = Metadata {
        name: Some(String::from("Astroport Lockdrop NFT - Lv. 2")),
        description: Some(String::from("You have participated in both phases of the Astroport lockdrop")),
        external_url: Some(String::from("https://astroport.fi")),
        image: Some(String::from("ipfs://hash_2")),
        attributes: Some(vec![
            Trait {
                display_type: None,
                trait_type: String::from("protocol"),
                value: String::from("astroport"),
            },
            Trait {
                display_type: None,
                trait_type: String::from("level"),
                value: String::from("2"),
            },
        ]),
        image_data: None,
        background_color: None,
        animation_url: None,
        youtube_url: None,
    };

    let metadata_lv_3 = Metadata {
        name: Some(String::from("Astroport Lockdrop NFT - Lv. 3")),
        description: Some(String::from("You have locked for maximum time in Astroport lockdrop phase 1")),
        external_url: Some(String::from("https://astroport.fi")),
        image: Some(String::from("ipfs://hash_3")),
        attributes: Some(vec![
            Trait {
                display_type: None,
                trait_type: String::from("protocol"),
                value: String::from("astroport"),
            },
            Trait {
                display_type: None,
                trait_type: String::from("level"),
                value: String::from("3"),
            },
        ]),
        image_data: None,
        background_color: None,
        animation_url: None,
        youtube_url: None,
    };

    InstantiateMsg {
        contract_info,
        minter: String::from("larry"),
        metadatas: vec![metadata_lv_1, metadata_lv_2, metadata_lv_3],
    }
}

fn create_mint_msgs() -> Vec<ExecuteMsg> {
    vec![
        ExecuteMsg::Mint {
            level: 1u8,
            owners: vec![String::from("alice"), String::from("bob"), String::from("larry")],
        },
        ExecuteMsg::Mint {
            level: 2u8,
            owners: vec![String::from("karen")],
        },
        ExecuteMsg::Mint {
            level: 3u8,
            owners: vec![String::from("larry"), String::from("chad")],
        },
    ]
}

fn setup_test() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
    let mut deps = mock_dependencies(&[]);

    instantiate(deps.as_mut(), mock_env(), mock_info("larry", &[]), create_init_msg()).unwrap();

    deps
}

fn query_helper<T: DeserializeOwned>(deps: Deps, msg: QueryMsg) -> T {
    from_binary(&query(deps, mock_env(), msg).unwrap()).unwrap()
}

#[test]
fn proper_instantiation() {
    let deps = setup_test();

    let res: ContractInfoResponse = query_helper(deps.as_ref(), QueryMsg::ContractInfo {});
    assert_eq!(res, create_init_msg().contract_info);
}

#[test]
fn minting() {
    let mut deps = setup_test();

    let msgs = create_mint_msgs();

    // Non-minter cannot mint
    let err = execute(deps.as_mut(), mock_env(), mock_info("jake", &[]), msgs[0].clone()).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Minter can mint
    for msg in msgs {
        execute(deps.as_mut(), mock_env(), mock_info("larry", &[]), msg).unwrap();
    }

    let res: NumTokensResponse = query_helper(deps.as_ref(), QueryMsg::NumTokens {});
    assert_eq!(res.count, 6u64);

    let res: TokensResponse = query_helper(
        deps.as_ref(),
        QueryMsg::AllTokens {
            start_after: None,
            limit: None,
        },
    );
    assert_eq!(res.tokens, vec!["1", "2", "3", "4", "5", "6"]);

    let res: TokensResponse = query_helper(
        deps.as_ref(),
        QueryMsg::Tokens {
            owner: String::from("larry"),
            start_after: None,
            limit: None,
        },
    );
    assert_eq!(res.tokens, vec!["3", "5"]);
}

#[test]
fn querying_nft_info() {
    let mut deps = setup_test();

    let msgs = create_mint_msgs();
    for msg in msgs {
        execute(deps.as_mut(), mock_env(), mock_info("larry", &[]), msg).unwrap();
    }

    let metadatas = create_init_msg().metadatas;

    let res: NftInfoResponse<Metadata> = query_helper(
        deps.as_ref(),
        QueryMsg::NftInfo {
            token_id: String::from("1"),
        },
    );
    assert_eq!(
        res,
        NftInfoResponse {
            token_uri: None,
            extension: metadatas[0].clone()
        }
    );

    let res: NftInfoResponse<Metadata> = query_helper(
        deps.as_ref(),
        QueryMsg::NftInfo {
            token_id: String::from("4"),
        },
    );
    assert_eq!(
        res,
        NftInfoResponse {
            token_uri: None,
            extension: metadatas[1].clone()
        }
    );

    let res: AllNftInfoResponse<Metadata> = query_helper(
        deps.as_ref(),
        QueryMsg::AllNftInfo {
            token_id: String::from("5"),
            include_expired: None,
        },
    );
    assert_eq!(
        res,
        AllNftInfoResponse {
            access: OwnerOfResponse {
                owner: String::from("larry"),
                approvals: vec![],
            },
            info: NftInfoResponse {
                token_uri: None,
                extension: metadatas[2].clone()
            }
        }
    );
}
