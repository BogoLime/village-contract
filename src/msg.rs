use schemars::JsonSchema;
use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::{Item};
use serde::{Deserialize, Serialize};
use crate::state::{*};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub storeName: String,
    pub refundPeriodPolicy: Option<u64>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Register{user: String},
    ListItem { 
        name:String,
        price:Coin,
        desc:Option<String>,
    },
    UpdateListing {
        id:u64,
        name:Option<String>,
        price:Option<Coin>,
        desc:Option<String>,
    },
    DeleteListing {
        id:u64,
    },
    Buy {
        id:u64,
    },
    // Refund {

    // },
    RateStore {
        rating:u16
    },
    RateSeller {
        sellerAddr:Addr,
        rating: u16
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetListing returns the current listing as a json-encoded number
    QueryConfig {},
    QueryStoreInfo{},
    QueryUser{id:Addr},
    QueryListing{id:u64},
    QueryListings {start_after: Option<u64>, limit: Option<u32>},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub config: Config
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StoreInfoResponse {
    pub store_info: Store
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserResponse {
    pub user_info: User
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ListingResponse {
    pub listing: Listing
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ListingsResponse {
    pub listings: Vec<Listing>
}

