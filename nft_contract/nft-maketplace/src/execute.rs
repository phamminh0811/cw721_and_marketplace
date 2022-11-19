
use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Response, Uint128, from_binary, Addr, Coin, CosmosMsg, coin, BankMsg, WasmMsg, to_binary};
use cw721::{Cw721ExecuteMsg, Cw721ReceiveMsg};
use nft_base::msg::CollectionInfoResponse;
use nft_base::QueryMsg as NFTQueryMsg;


use crate::error::ContractError;
use crate::state::{CONTRACT_INFO, ADMIN, NFT_CONTRACTS, SaleType, increment_offerings, Offering, OFFERINGS, BidOffering, BID_OFFERINGS};


pub fn exec_add_nft_contract(
    deps: DepsMut, 
    _env: Env, 
    info: MessageInfo, 
    address: String
) -> Result<Response, ContractError> {
    let admin = ADMIN.load(deps.storage)?;
    if admin != info.sender {
        return Err(ContractError::Unauthorized{});
    }

    let mut nft_contracts = NFT_CONTRACTS.load(deps.storage)?;
    let nft_contract = deps.api.addr_validate(&address)?;
    
    if !nft_contracts.contains(&nft_contract) {
        nft_contracts.push(nft_contract);
        NFT_CONTRACTS.save(deps.storage, &nft_contracts)?;
    }
    Ok(Response::new()
        .add_attribute("action", "add_code_id")
    )
}

pub fn exec_make_offer(
    deps: DepsMut, 
    _env: Env, 
    info: MessageInfo, 
    offering_id: String
) -> Result<Response, ContractError> {
    let offer = OFFERINGS.load(deps.storage, &offering_id)?;
    if let SaleType::FixedPrice(price)  = offer.sale_type {
        let royalty_info = offer.royalty_info;

        let funds_from_sender = one_coin(&deps.as_ref(), &info)?;
        let Coin {amount, denom} = funds_from_sender;
        if amount < price {
            return Err(ContractError::InsufficientDeposit {});
        }
        
        let mut cosmos_msg:Vec<CosmosMsg> = vec![];
        let royalty_fee;
        let net_price;
    
        if royalty_info.is_some() {
            let creator_address = royalty_info.clone().unwrap().payment_address;
            let share = royalty_info.unwrap().share;
    
            royalty_fee = amount * share;
            net_price = amount - amount * share;
    
            // send price to creator
            let transfer_royalty_fee_msg = CosmosMsg::Bank(BankMsg::Send {
                to_address: creator_address,
                amount: vec![coin(royalty_fee.u128(), denom.clone())],
            });
    
            cosmos_msg.push(transfer_royalty_fee_msg);
        
        }else {
            royalty_fee = Uint128::from(0u128);
            net_price = amount;
        }

        let transfer_cw721_msg = Cw721ExecuteMsg::TransferNft {
            recipient: info.sender.clone().into_string(),
            token_id: offer.token_id.clone(),
        };
    
        let exec_cw721_transfer = WasmMsg::Execute {
            contract_addr: (&offer.nft_address).to_string(),
            msg: to_binary(&transfer_cw721_msg)?,
            funds: vec![],
        };
        let cw721_transfer_cosmos_msg: CosmosMsg =  exec_cw721_transfer.into();
        // send price to seller
        let transfer_net_price_msg: CosmosMsg = CosmosMsg::Bank(BankMsg::Send {
            to_address: (&offer.seller).to_string(),
            amount: vec![coin(net_price.u128(), denom.clone())],
        }).into();

        cosmos_msg.push(transfer_net_price_msg);
        cosmos_msg.push(cw721_transfer_cosmos_msg);

        OFFERINGS.remove(deps.storage, &offering_id);

        let price_string = format!("{} {}", amount.clone(), denom);
        Ok(Response::new()
            .add_messages(cosmos_msg)
            .add_attribute("action", "make_order")
            .add_attribute("seller", offer.seller.to_string())
            .add_attribute("buyer", info.sender)
            .add_attribute("paid_price", price_string)
            .add_attribute("token_id", offer.token_id)
            .add_attribute("contract_addr", offer.nft_address.to_string())
            .add_attribute("net_price", net_price)
            .add_attribute("royalty_fee", royalty_fee))
    }  else {
        Err(ContractError::SaleTypeMustBeFixedPrice {})
    }
}

pub fn exec_bid(
    deps: DepsMut, 
    env: Env, 
    info: MessageInfo, 
    offering_id: String,
) -> Result<Response, ContractError> {
    let offer = OFFERINGS.load(deps.storage, &offering_id)?;
    if let SaleType::Auction(bid)  = offer.sale_type {
        if bid.expiration.is_expired(&env.block) {
            return Err(ContractError::BidExpiration {});
        }
        let mut bid_offering = BID_OFFERINGS.load(deps.storage, &offering_id)?;

        let funds_from_sender = one_coin(&deps.as_ref(), &info)?;
        let Coin {amount, denom} = funds_from_sender;
        
        let mut cosmos_msg:Vec<CosmosMsg> = vec![];

        if bid_offering.highest_bid_price.is_none(){
            if bid.start_price <= amount{
                bid_offering.new(amount, info.sender.clone());
            } else {return Err(ContractError::InsufficientDeposit {  });}
        } else{
            let highest_price = bid_offering.highest_price();
            let increase_per_bid = bid.increase_per_bid.unwrap_or(Uint128::from(0u128));
            if  highest_price + increase_per_bid < amount{
                let transfer_bid_msg: CosmosMsg = CosmosMsg::Bank(BankMsg::Send {
                    to_address: (&bid_offering.clone().address.unwrap()).to_string(),
                    amount: vec![coin(highest_price.u128(), denom.clone())],
                }).into();
                cosmos_msg.push(transfer_bid_msg);
                bid_offering.new(amount, info.sender.clone());
            } else {return Err(ContractError::InsufficientDeposit {  });}
        }
        let price_string = format!("{} {}", amount, denom);
        Ok(Response::new()
            .add_messages(cosmos_msg)
            .add_attribute("action", "bid")
            .add_attribute("bidder", info.sender)
            .add_attribute("price",price_string)
            .add_attribute("token_id", offer.token_id)
            .add_attribute("contract_addr", offer.nft_address.to_string()))
    } else {
        Err(ContractError::SaleTypeMustBeAuction { })
    }
    
}

pub fn exec_close_bid(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    offering_id: String
) -> Result<Response, ContractError> {
    let offer = OFFERINGS.load(deps.storage, &offering_id)?;
    
    if let SaleType::Auction(bid)  = offer.sale_type {
        let bid_offering = BID_OFFERINGS.load(deps.storage, &offering_id)?;

        if info.sender == offer.seller {
            let transfer_cw721_msg = Cw721ExecuteMsg::TransferNft {
                recipient: offer.seller.clone().into_string(),
                token_id: offer.token_id.clone(),
            };
    
            let exec_cw721_transfer = WasmMsg::Execute {
                contract_addr: offer.nft_address.clone().into_string(),
                msg: to_binary(&transfer_cw721_msg)?,
                funds: vec![],
            };
    
            let cw721_transfer_cosmos_msg: Vec<CosmosMsg> = vec![exec_cw721_transfer.into()];
            OFFERINGS.remove(deps.storage, &offering_id);
    
            return Ok(Response::new().add_messages(cw721_transfer_cosmos_msg)
                .add_attribute("action", "seller_close_bid")
                .add_attribute("seller", info.sender)
                .add_attribute("offering_id", offering_id))
        } else if (Some(info.sender.clone()) == bid_offering.address) && bid.expiration.is_expired(&env.block){
            let royalty_info = offer.royalty_info;
            let amount = bid_offering.highest_price();
            let denom = CONTRACT_INFO.load(deps.storage)?.native_denom;

            let mut cosmos_msg:Vec<CosmosMsg> = vec![];
            let royalty_fee;
            let net_price;
        
            if royalty_info.is_some() {
                let creator_address = royalty_info.clone().unwrap().payment_address;
                let share = royalty_info.unwrap().share;
        
                royalty_fee = amount * share;
                net_price = amount - amount * share;
        
                // send price to creator
                let transfer_royalty_fee_msg = CosmosMsg::Bank(BankMsg::Send {
                    to_address: creator_address,
                    amount: vec![coin(royalty_fee.u128(), denom.clone())],
                });
        
                cosmos_msg.push(transfer_royalty_fee_msg);
            
            }else {
                royalty_fee = Uint128::from(0u128);
                net_price = amount;
            }

            let transfer_cw721_msg = Cw721ExecuteMsg::TransferNft {
                recipient: info.sender.clone().into_string(),
                token_id: offer.token_id.clone(),
            };
        
            let exec_cw721_transfer = WasmMsg::Execute {
                contract_addr: (&offer.nft_address).to_string(),
                msg: to_binary(&transfer_cw721_msg)?,
                funds: vec![],
            };
            let cw721_transfer_cosmos_msg: CosmosMsg =  exec_cw721_transfer.into();
            // send price to seller
            let transfer_net_price_msg: CosmosMsg = CosmosMsg::Bank(BankMsg::Send {
                to_address: (&offer.seller).to_string(),
                amount: vec![coin(net_price.u128(), denom.clone())],
            }).into();
    
            cosmos_msg.push(transfer_net_price_msg);
            cosmos_msg.push(cw721_transfer_cosmos_msg);
    
            OFFERINGS.remove(deps.storage, &offering_id);

            let price_string = format!("{} {}", amount, denom);
            Ok(Response::new()
                .add_messages(cosmos_msg)
                .add_attribute("action", "make_order")
                .add_attribute("seller", offer.seller.to_string())
                .add_attribute("buyer", info.sender)
                .add_attribute("paid_price", price_string)
                .add_attribute("token_id", offer.token_id)
                .add_attribute("contract_addr", offer.nft_address.to_string())
                .add_attribute("net_price", net_price)
                .add_attribute("royalty_fee", royalty_fee))
        } else {
            Err(ContractError::Unauthorized {  })
        }
    } else {
        Err(ContractError::SaleTypeMustBeAuction { })
    }
}

pub fn exec_update_price(
    deps: DepsMut, 
    _env: Env, 
    info: MessageInfo, 
    offering_id: String,
    update_price: Uint128
) -> Result<Response, ContractError> {
    let mut offer = OFFERINGS.load(deps.storage, &offering_id)?;
    if let SaleType::FixedPrice(_) = offer.sale_type {
        if info.sender != offer.seller {
            return Err(ContractError::Unauthorized {});
        }
        if update_price.is_zero() {
            return Err(ContractError::PriceMustBePosiTive {});
        }
        offer.sale_type = SaleType::FixedPrice(update_price);
        OFFERINGS.save(deps.storage, &offering_id, &offer)?;
    
        let price_string = format!("{} {}", update_price, CONTRACT_INFO.load(deps.storage)?.native_denom);

        Ok(Response::new()
        .add_attribute("action", "update_price")
        .add_attribute("sender", info.sender)
        .add_attribute("offering_id", offering_id)
        .add_attribute("update_price", price_string))
    } else {
        Err(ContractError::SaleTypeMustBeFixedPrice { })
    }
}

pub fn exec_receive_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    rcv_msg: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    let nft_addresses = NFT_CONTRACTS.load(deps.storage)?;
    if !nft_addresses.contains(&info.sender){
        return Err(ContractError::NFTAddressNotMatch {  })
    } 
    let msg: SaleType = from_binary(&rcv_msg.msg)?;
    let token_id: String = rcv_msg.token_id;
    let seller: Addr = deps.api.addr_validate(&rcv_msg.sender)?;

    let id = increment_offerings(deps.storage)?.to_string();
    let collection_info: CollectionInfoResponse = deps
        .querier
        .query_wasm_smart(info.sender.clone(), &NFTQueryMsg::CollectionInfo {})?;
    
    let royalty_info = collection_info.royalty_info;

    match msg {
        SaleType::FixedPrice(price) => {
            if price.is_zero() {
                return Err(ContractError::PriceMustBePosiTive {});
            }
        },
        SaleType::Auction(_) => {
            let bid_offering = BidOffering::default(&env);
            BID_OFFERINGS.save(deps.storage, &id, &bid_offering)?;
        },
    }
    let offer = Offering {
        token_id,
        nft_address : info.sender.clone(),
        royalty_info: royalty_info.clone(),
        seller,
        sale_type: msg,
        listing_time: env.block.time,
    };
    OFFERINGS.save(deps.storage, &id, &offer)?;
        Ok(Response::new()
        .add_attribute("action", "create_sale")
        .add_attribute("original_contract", info.sender)
        .add_attribute("seller", offer.seller.to_string())
        .add_attribute("token_id", offer.token_id))
}

pub fn exec_withdraw_nft(
    deps: DepsMut, 
    _env: Env, 
    info: MessageInfo, 
    offering_id: String
) -> Result<Response, ContractError> {
    
    let offer = OFFERINGS.load(deps.storage, &offering_id)?;
    if let SaleType::FixedPrice(_) = offer.sale_type {
        if info.sender != offer.seller {
            return Err(ContractError::Unauthorized {});
        }

        let transfer_cw721_msg = Cw721ExecuteMsg::TransferNft {
            recipient: offer.seller.clone().into_string(),
            token_id: offer.token_id.clone(),
        };

        let exec_cw721_transfer = WasmMsg::Execute {
            contract_addr: offer.nft_address.clone().into_string(),
            msg: to_binary(&transfer_cw721_msg)?,
            funds: vec![],
        };

        let cw721_transfer_cosmos_msg: Vec<CosmosMsg> = vec![exec_cw721_transfer.into()];
        OFFERINGS.remove(deps.storage, &offering_id);

        Ok(Response::new().add_messages(cw721_transfer_cosmos_msg)
            .add_attribute("action", "withdraw_nft")
            .add_attribute("seller", info.sender)
            .add_attribute("offering_id", offering_id))
    } else {
        Err(ContractError::SaleTypeMustBeFixedPrice {  })
    }
}


pub fn one_coin(deps: &Deps, info: &MessageInfo) -> Result<Coin, ContractError> {
    match info.funds.len() {
        0 => Err(ContractError::NoFunds {}),
        1 => {
            let coin = &info.funds[0];
            if coin.amount.is_zero() {
                Err(ContractError::NoFunds {})
            } else if coin.denom == CONTRACT_INFO.load(deps.storage)?.native_denom {
                Err(ContractError::DenomNotMatch {})
            } else {
                Ok(coin.clone())
            }
        }
        _ => Err(ContractError::MultipleDenoms {}),
    }
}