#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::execute::{exec_add_nft_contract, exec_withdraw_nft, exec_make_offer, exec_bid, exec_close_bid, exec_update_price, exec_receive_nft};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{ContractInfo, CONTRACT_INFO, ADMIN, NFT_CONTRACTS};


const CONTRACT_NAME: &str = "crates.io:maketplace";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let admin = msg.admin.unwrap_or_default();
    let validate_admin = deps.api.addr_validate(&admin).unwrap_or(info.sender);
    ADMIN.save(deps.storage, &validate_admin)?;
    let info = ContractInfo { name: msg.name, native_denom: msg.native_denom};
    CONTRACT_INFO.save(deps.storage, &info)?;

    let mut nft_contracts= vec![];
    for address in &msg.nft_contracts {
        let nft_contract = deps.api.addr_validate(&address)?;
        if !nft_contracts.contains(&nft_contract) {
            nft_contracts.push(nft_contract);
        }
    }
    NFT_CONTRACTS.save(deps.storage, &nft_contracts)?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("name", info.name)
        .add_attribute("admin", validate_admin.to_string())
        )
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AddNFTContract { address } => exec_add_nft_contract(deps, env, info, address), 
        ExecuteMsg::WithdrawNft { offering_id } => exec_withdraw_nft(deps, env, info, offering_id),
        ExecuteMsg::MakeOffer { offering_id } => exec_make_offer(deps, env, info, offering_id),
        ExecuteMsg::Bid { offering_id } => exec_bid(deps, env, info, offering_id),
        ExecuteMsg::CloseBid { offering_id } => exec_close_bid(deps, env, info, offering_id),
        ExecuteMsg::UpdatePrice { offering_id, update_price } => exec_update_price(deps, env, info, offering_id, update_price),
        ExecuteMsg::ReceiveNft(msg) => exec_receive_nft(deps, env, info, msg)
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}

#[cfg(test)]
mod tests {}
