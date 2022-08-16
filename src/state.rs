use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Addr, Coin, Decimal};

use cw_storage_plus::{Item, Map};


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Listing {
    pub name:String,
    pub price:Coin,
    pub seller: Addr,
    pub desc:Option<String>,
    pub id:u64
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct User {
    pub name: String,
    pub role: Roles,
    pub rating: Option<Rating>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Store {
    pub storeName: String,
    pub storeRating: Rating,
    pub storeAdmin: Addr,
    pub refundPeriodPolicy: u64
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Rating {
    pub rating:String,
    pub votes:String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Roles {
    Admin,
    Regular
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CountedValues{
    Listing,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub payDenom: String,
}



pub const CONFIG: Item<Config> = Item::new("config");
pub const LISTINGS: Map<u64, Listing> = Map::new("listings");
pub const USERS: Map<&Addr, User> = Map::new("users");
pub const STORE: Item<Store> = Item::new("store");
pub const STORE_RATERS: Map<Addr,bool> = Map::new("store_raters");
pub const SELLER_RATERS: Map<Addr,bool> = Map::new("seller_raters");

// ID COUNTERS
pub const ITEM_COUNTER: Item<u64> = Item::new("item_counter");

