use cosmwasm_std::Uint128;
use cw721::Cw721ReceiveMsg;
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin: Option<String>,
    pub name: String,
    pub native_denom: String,
    pub nft_contracts: Vec<String>,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    AddNFTContract { address: String},
    WithdrawNft { offering_id: String},
    MakeOffer { offering_id: String},
    Bid{ offering_id: String },
    CloseBid { offering_id: String},
    UpdatePrice { offering_id: String, update_price: Uint128},
    ReceiveNft(Cw721ReceiveMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    
}
