use cosmwasm_std::Timestamp;
use nft_base::msg::RoyaltyInfoResponse;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::SaleType;

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ContractInfoResponse {
    pub name: String,
    pub native_denom: String
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct QueryOfferingsResult {
    pub id: String,
    pub token_id: String,
    pub sale_type: SaleType,
    pub royalty_info: Option<RoyaltyInfoResponse>,
    pub nft_address: String,
    pub seller: String,
    pub listing_time: Timestamp,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OfferingsResponse {
    pub offerings: Vec<QueryOfferingsResult>,
}