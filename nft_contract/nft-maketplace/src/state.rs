use cosmwasm_std::{Addr, Uint128, Timestamp, Storage, StdResult, Env};
use cw721::Expiration;
use nft_base::msg::RoyaltyInfoResponse;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use cw_storage_plus::{Item, Map};
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ContractInfo {
    pub name: String,
    pub native_denom: String
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub enum SaleType {
    FixedPrice(Uint128),
    Auction(Bid)
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Bid {
    pub start_price: Uint128,
    pub increase_per_bid: Option<Uint128>,
    pub expiration: Expiration,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Offering {
    pub token_id: String,
    pub nft_address: Addr,
    pub royalty_info: Option<RoyaltyInfoResponse>,
    pub seller: Addr,
    pub sale_type: SaleType,
    pub listing_time: Timestamp,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct BidOffering{
    pub highest_bid_price: Option<Uint128>,
    pub address: Option<Addr>,
    pub start_timestamp: Timestamp
}

impl BidOffering {
    pub fn default(env: &Env)-> Self {
        Self {
            highest_bid_price: None,
            address: None,
            start_timestamp: env.block.time
        }
    }

    pub fn new(&mut self, price: Uint128, address: Addr){
        self.highest_bid_price = Some(price);
        self.address = Some(address);
    }

    pub fn highest_price(&self) -> Uint128{
        self.highest_bid_price.unwrap()
    }
}
pub const OFFERINGS: Map<&str, Offering> = Map::new("offerings");
pub const OFFERINGS_COUNT: Item<u64> = Item::new("num_offerings");
pub const BID_OFFERINGS: Map<&str, BidOffering> = Map::new("bid_offerings");
pub const CONTRACT_INFO: Item<ContractInfo> = Item::new("contract_info");
pub const ADMIN: Item<Addr> = Item::new("admin");
pub const NFT_CONTRACTS: Item<Vec<Addr>>= Item::new("nft_contracts");

pub fn num_offerings(storage: &dyn Storage) -> StdResult<u64> {
    Ok(OFFERINGS_COUNT.may_load(storage)?.unwrap_or_default())
}

pub fn increment_offerings(storage: &mut dyn Storage) -> StdResult<u64> {
    let val = num_offerings(storage)? + 1;
    OFFERINGS_COUNT.save(storage, &val)?;

    Ok(val)
}