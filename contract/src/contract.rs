use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult,
};
use cw721::{AllNftInfoResponse, Cw721Execute, Cw721Query, NftInfoResponse};
use cw721_base::{state::TokenInfo, ContractError, Cw721Contract};
use cw721_metadata_onchain::Metadata;

use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::METADATA;

// We extend the generic CW721 contract
// For each token, we only store which level it is, specified by an `u8` which can take on the values of 1, 2, 3
pub type Parent<'a> = Cw721Contract<'a, u8, Empty>;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let parent = Parent::default();
    let minter_addr = deps.api.addr_validate(&msg.minter)?;

    parent.contract_info.save(deps.storage, &msg.contract_info)?;
    parent.token_count.save(deps.storage, &0)?;
    parent.minter.save(deps.storage, &minter_addr)?;

    for (i, metadata) in msg.metadatas.iter().enumerate() {
        METADATA.save(deps.storage, (i as u8 + 1).into(), metadata)?;
    }

    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let parent = Parent::default();

    match msg {
        // Commands specific to this NFT
        ExecuteMsg::Mint {
            level,
            owners,
        } => execute_mint(deps, info, level, owners),
        ExecuteMsg::UpdateMinter {
             new_minter
        } => execute_update_minter(deps, info, new_minter),

        // Generic CW721 commands; we simply dispatch them to parent
        ExecuteMsg::TransferNft {
            recipient,
            token_id,
        } => parent.transfer_nft(deps, env, info, recipient, token_id),
        ExecuteMsg::SendNft {
            contract,
            token_id,
            msg,
        } => parent.send_nft(deps, env, info, contract, token_id, msg),
        ExecuteMsg::Approve {
            spender,
            token_id,
            expires,
        } => parent.approve(deps, env, info, spender, token_id, expires),
        ExecuteMsg::Revoke {
            spender,
            token_id,
        } => parent.revoke(deps, env, info, spender, token_id),
        ExecuteMsg::ApproveAll {
            operator,
            expires,
        } => parent.approve_all(deps, env, info, operator, expires),
        ExecuteMsg::RevokeAll {
            operator,
        } => parent.revoke_all(deps, env, info, operator),
    }
}

pub fn execute_mint(
    deps: DepsMut,
    info: MessageInfo,
    level: u8,
    owners: Vec<String>,
) -> Result<Response, ContractError> {
    let parent = Parent::default();

    let minter_addr = parent.minter.load(deps.storage)?;
    if info.sender != minter_addr {
        return Err(ContractError::Unauthorized {});
    }
    
    let mut token_count = parent.token_count.load(deps.storage)?;
    for owner in &owners {
        token_count += 1; // No need for safe math. No way we're gonna mint more than 2^64 tokens lol
        parent.tokens.save(
            deps.storage,
            &token_count.to_string(),
            &TokenInfo {
                owner: deps.api.addr_validate(owner)?,
                approvals: vec![],
                token_uri: None,
                extension: level,
            },
        )?;
    }

    parent.token_count.save(deps.storage, &token_count)?;

    Ok(Response::new()
        .add_attribute("action", "lockdrop-nft/execute/mint")
        .add_attribute("minter", info.sender)
        .add_attribute("num_minted", owners.len().to_string()))
}

pub fn execute_update_minter(
    deps: DepsMut,
    info: MessageInfo,
    new_minter: String,
) -> Result<Response, ContractError> {
    let parent = Parent::default();

    let minter_addr = parent.minter.load(deps.storage)?;
    if info.sender != minter_addr {
        return Err(ContractError::Unauthorized {});
    }

    let new_minter_addr = deps.api.addr_validate(&new_minter)?;
    parent.minter.save(deps.storage, &new_minter_addr)?;

    Ok(Response::new()
        .add_attribute("action", "lockdrop-nft/execute/update_minter")
        .add_attribute("current_minter", minter_addr)
        .add_attribute("new_minter", new_minter_addr))
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let parent = Parent::default();

    match msg {
        // We create special logics for the following queries
        QueryMsg::NftInfo {
            token_id,
        } => to_binary(&query_nft_info(deps, token_id)?),
        QueryMsg::AllNftInfo {
            token_id,
            include_expired,
        } => to_binary(&query_all_nft_info(deps, env, token_id, include_expired.unwrap_or(false))?),

        // Generic CW721 queries; we simply dispatch them to parent
        QueryMsg::ContractInfo {} => to_binary(&parent.contract_info(deps)?),
        QueryMsg::Minter {} => to_binary(&parent.minter(deps)?),
        QueryMsg::NumTokens {} => to_binary(&parent.num_tokens(deps)?),
        QueryMsg::OwnerOf {
            token_id,
            include_expired,
        } => to_binary(&parent.owner_of(deps, env, token_id, include_expired.unwrap_or(false))?),
        QueryMsg::ApprovedForAll {
            owner,
            include_expired,
            start_after,
            limit,
        } => to_binary(&parent.all_approvals(
            deps,
            env,
            owner,
            include_expired.unwrap_or(false),
            start_after,
            limit,
        )?),
        QueryMsg::Tokens {
            owner,
            start_after,
            limit,
        } => to_binary(&parent.tokens(deps, owner, start_after, limit)?),
        QueryMsg::AllTokens {
            start_after,
            limit,
        } => to_binary(&parent.all_tokens(deps, start_after, limit)?),
    }
}

pub fn query_nft_info(deps: Deps, token_id: String) -> StdResult<NftInfoResponse<Metadata>> {
    let parent = Parent::default();

    let level = parent.tokens.load(deps.storage, &token_id)?.extension;
    let metadata = METADATA.load(deps.storage, level.into())?;

    Ok(NftInfoResponse {
        token_uri: None,
        extension: metadata,
    })
}

pub fn query_all_nft_info(
    deps: Deps,
    env: Env,
    token_id: String,
    include_expired: bool,
) -> StdResult<AllNftInfoResponse<Metadata>> {
    let parent = Parent::default();

    Ok(AllNftInfoResponse {
        access: parent.owner_of(deps, env, token_id.clone(), include_expired)?,
        info: query_nft_info(deps, token_id)?,
    })
}

#[entry_point]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default()) // do nothing
}
