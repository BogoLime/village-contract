#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary,Order, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Addr,Coin};
use cw2::set_contract_version;
use cosmwasm_std::{Uint128, Decimal};

use crate::error::ContractError;
use crate::msg::{*};
use crate::state::{Listing,LISTINGS,User, USERS, 
    Store, STORE, Roles, Config, 
    CONFIG, ITEM_COUNTER,Rating, STORE_RATERS, SELLER_RATERS};
use crate::helpers::{*};
use cw_storage_plus::Bound;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:village-contracts";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    validate_name(&msg.storeName)?;

    let config_state = Config {
        payDenom: String::from("ucosm"),
    };
    
    let state = Store {
        storeName:msg.storeName.clone(),
        storeRating: Rating {
            rating:String::from("0.00"),
            votes:String::from("0.00")
        },
        storeAdmin: info.sender.clone(),
        refundPeriodPolicy:msg.refundPeriodPolicy.unwrap_or(100),
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    STORE.save(deps.storage, &state)?;
    CONFIG.save(deps.storage, &config_state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("storeName", msg.storeName))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Register {user} => execute_register(deps,info,user),
        ExecuteMsg::ListItem {name, price, desc} => execute_new_listing(deps,info,name,price,desc),
        ExecuteMsg::UpdateListing {id,name,price,desc} => execute_update_listing(deps,info,id,name,price,desc),
        ExecuteMsg::DeleteListing {id} => execute_delete_listing(deps,info,id),
        ExecuteMsg::Buy {id} => execute_buy(deps,info,id),
        ExecuteMsg::RateStore {rating} => execute_rate_store(deps,info,rating),
        ExecuteMsg::RateSeller {sellerAddr,rating} => execute_rate_seller(deps,info,sellerAddr,rating),
    }
}

pub fn execute_register(
    deps: DepsMut, 
    info: MessageInfo, 
    userName:String
) -> Result<Response, ContractError>{
    validate_name(&userName)?;

    let isValid = deps.api.addr_validate(&info.sender.to_string())?;
    let mut id:Addr;

    id = isValid;

    let isRegistered = USERS.load(deps.storage,&id);

    if isRegistered.is_ok(){
        Err(ContractError::ExistingUser{})?;
    }

    let user = User{
        name:userName,
        role:Roles::Regular,
        rating:None
    };
    
    USERS.save(deps.storage,&id,&user)?;

    Ok(Response::new()
        .add_attribute("method","register")
        .add_attribute("user_id", id.to_string())
    )
}

pub fn execute_new_listing(
    deps: DepsMut, 
    info: MessageInfo, 
    name:String, 
    price:Coin, 
    desc:Option<String>
) -> Result<Response, ContractError>{
    let config_state = CONFIG.load(deps.storage)?;
    validate_name(&name)?;
    check_for_valid_coin(&price,config_state.payDenom)?;
    let isRegistered = USERS.load(deps.storage,&info.sender).is_ok();

    if !isRegistered{
        return Err(ContractError::GenericMsgErr
            {msg:String::from("You haven't registered")});
    }

    let id = update_counter(deps.storage, &ITEM_COUNTER)?;

    let listing = Listing {
        name,
        price,
        desc,
        id,
        seller: info.sender
    };

    LISTINGS.save(deps.storage, id, &listing)?;

    Ok(Response::new()
        .add_attribute("method", "add_new")
        .add_attribute("listing_id", id.to_string())
    )
}

pub fn execute_update_listing(
    deps: DepsMut, 
    info: MessageInfo, 
    id:u64, 
    name:Option<String>, 
    price:Option<Coin>,
    desc:Option<String>
    ) -> Result<Response, ContractError>{
    let config_state = CONFIG.load(deps.storage)?;

    if name.is_some(){
        let name = name.clone();
        validate_name(&name.unwrap())?;
    }
    if price.is_some(){
        let price = price.clone();
        check_for_valid_coin(&price.unwrap(), config_state.payDenom)?;
    }
    
    let listing = LISTINGS.load(deps.storage, id)?;

    let updated_listing = Listing {
        name:name.unwrap_or(listing.name),
        price:price.unwrap_or(listing.price),
        desc,
        id,
        seller: info.sender
    };

    LISTINGS.save(deps.storage, id, &updated_listing)?;

    Ok(Response::new()
    .add_attribute("method", "update")
    .add_attribute("listing_id", id.to_string())
)


}


pub fn execute_delete_listing(
    deps: DepsMut, 
    info: MessageInfo, 
    id:u64
) -> Result<Response, ContractError>{
    let listing = LISTINGS.load(deps.storage,id)?;

    if listing.seller != info.sender {
        return Err(ContractError::NotAnOwner{});
    };

    LISTINGS.remove(deps.storage, id);
    Ok(Response::new()
    .add_attribute("method", "delete")
    .add_attribute("listing_id", id.to_string())
)

}

pub fn execute_buy(
    deps: DepsMut, 
    info: MessageInfo, 
    id:u64
) -> Result<Response, ContractError>{
    let listing = LISTINGS.load(deps.storage,id)?;

    if listing.seller == info.sender {
        return Err(ContractError::BuyFromSelf{});
    };

    if info.funds.len() < 1 {
        return Err(ContractError::GenericMsgErr
            {msg:String::from("Send some tokens with the Trx.")});
    }
    let config_state = CONFIG.load(deps.storage)?;
    check_for_valid_coin(&info.funds[0], config_state.payDenom)?;

    let rec = u128::from(info.funds[0].amount);
    let exp = u128::from(listing.price.amount);

    if rec < exp {
        return Err(ContractError::InsufficientFunds{exp,rec});
    }

    // Remove item, that has been sold
    LISTINGS.remove(deps.storage,id);

    Ok(Response::new()
    .add_attribute("method", "buy")
    .add_attribute("listing_id", id.to_string())
    .add_attribute("buyer", info.sender.to_string())
)

}


pub fn execute_rate_store(
    deps: DepsMut, 
    info: MessageInfo, 
    rating:u16
) -> Result<Response, ContractError>{
    let store = STORE.load(deps.storage)?;
    let hasRated = STORE_RATERS.load(deps.storage,info.sender.clone()).is_ok();

    if hasRated {
        return Err(ContractError::HasAlreadyRated{addr:info.sender.clone()});
    };

    let curRating = store.storeRating;

    let calculated =  match calculate_rating(deps.storage,rating,&curRating){
        Ok(res) => res,
        Err(err) => return Err(ContractError::GenericMsgErr{msg:String::from("Failed calculating rating")})
    };

    let updated = Store{
        storeName: store.storeName,
        storeRating:calculated,
        storeAdmin: store.storeAdmin,
        refundPeriodPolicy: store.refundPeriodPolicy,
    };

    STORE.save(deps.storage,&updated)?;
    STORE_RATERS.save(deps.storage, info.sender.clone(), &bool::from(true))?;

    Ok(Response::new()
    .add_attribute("method", "rate_store")
    .add_attribute("rater", info.sender.to_string())
)

}


pub fn execute_rate_seller(
    deps: DepsMut, 
    info: MessageInfo, 
    sellerAddr: Addr,
    rating:u16
) -> Result<Response, ContractError>{
    let user = USERS.load(deps.storage,&sellerAddr)?;
    let hasRated = SELLER_RATERS.load(deps.storage,info.sender.clone()).is_ok();

    if info.sender == sellerAddr {
        return Err(ContractError::GenericMsgErr{msg:String::from(" Can't rate yourself")})
    }

    if hasRated {
        return Err(ContractError::HasAlreadyRated{addr:info.sender});
    };

    let curRating = user.rating.unwrap_or(Rating{rating:String::from("0.00"),votes:String::from("0.00")});
    let newRating = calculate_rating(deps.storage,rating,&curRating).ok();

    let updated = User{
        name: user.name,
        role:user.role,
        rating: newRating,
    };

    Ok(Response::new()
    .add_attribute("method", "rate_seller")
    .add_attribute("rater", info.sender.to_string())
)

}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryConfig { } => {
            to_binary(&query_config(deps)?)
        },
        QueryMsg::QueryStoreInfo { } => {
            to_binary(&query_store_info(deps)?)
        },
        QueryMsg::QueryUser { id } => to_binary(&query_user(deps, id)?),
        QueryMsg::QueryListing { id } => to_binary(&query_entry(deps, id)?),
        QueryMsg::QueryListings { start_after, limit } => {
            to_binary(&query_list(deps, start_after, limit)?)
        },
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let entry = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        config:entry,
    })
}

fn query_store_info(deps: Deps) -> StdResult<StoreInfoResponse> {
    let entry = STORE.load(deps.storage)?;
    Ok(StoreInfoResponse {
        store_info:entry,
    })
}

fn query_user(deps: Deps, id:Addr) -> StdResult<UserResponse> {
    let entry = USERS.load(deps.storage,&id)?;
    Ok(UserResponse {
        user_info:entry,
    })
}

fn query_entry(deps: Deps, id: u64) -> StdResult<ListingResponse> {
    let entry = LISTINGS.load(deps.storage, id)?;
    Ok(ListingResponse {
        listing:entry
    })
}

// Limits for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

fn query_list(deps: Deps, start_after: Option<u64>, limit: Option<u32>) -> StdResult<ListingsResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);
    let entries: StdResult<Vec<_>> = LISTINGS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect();

    let result = ListingsResponse {
        listings: entries?.into_iter().map(|l| l.1).collect(),
    };
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, attr};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {storeName:String::from("village1"),refundPeriodPolicy:Some(50)};
        let info = mock_info("creator", &coins(10000, "ucosm"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::QueryStoreInfo {}).unwrap();
        let value: StoreInfoResponse = from_binary(&res).unwrap();
        assert_eq!("village1", value.store_info.storeName);
        assert_eq!(50, value.store_info.refundPeriodPolicy);
    }

    #[test]
    fn register() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {storeName:String::from("village1"),refundPeriodPolicy:Some(50)};
        let info = mock_info("creator", &coins(10000, "ucosm"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

       
        let info = mock_info("user-one", &coins(10000, "ucosm"));
        let msg = ExecuteMsg::Register {user:String::from("user1")};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(_res.attributes.len(),2);
        assert_eq!(_res.attributes[0],attr("method","register"));

        // Already registered - Fail
        let info = mock_info("user-one", &coins(10000, "ucosm"));
        let msg = ExecuteMsg::Register {user:String::from("user1")};
        let _res = execute(deps.as_mut(), mock_env(), info, msg);
        match _res {
            Err(ContractError::ExistingUser{}) => {}
            _ => panic!("Already registered user"),
        }

        // Invalid name - Fail
        let info = mock_info("user-one", &coins(10000, "ucosm"));
        let msg = ExecuteMsg::Register {user:String::from("User1")};
        let _res = execute(deps.as_mut(), mock_env(), info, msg);
        match _res {
            Err(ContractError::InvalidCharacter{c}) => {}
            _ => panic!("Invalid character"),
        }
    }


    #[test]
    fn new_listing_and_update() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {storeName:String::from("village1"),refundPeriodPolicy:Some(50)};
        let info = mock_info("creator", &coins(10000, "ucosm"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();


        // No registration - failed create
        let info = mock_info("user-one", &coins(10000, "ucosm"));
        let msg = ExecuteMsg::ListItem {
            name:"lap-top".to_string(),
            price:Coin{amount:Uint128::from(150 as u128),denom:"ucosm".to_string()},
            desc:Some("Very Nice Lap-Top".to_string()),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg);
        match _res {
            Err(ContractError::GenericMsgErr{msg}) => {}
            _ => panic!("You haven't registered"),
        }
        
        // Register user
        let info = mock_info("user-one", &coins(10000, "ucosm"));
        let msg = ExecuteMsg::Register {user:String::from("user1")};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(_res.attributes.len(),2);
        assert_eq!(_res.attributes[0],attr("method","register"));

        // Success create
        let info = mock_info("user-one", &coins(10000, "ucosm"));
        let msg = ExecuteMsg::ListItem {
            name:"lap-top".to_string(),
            price:Coin{amount:Uint128::from(150 as u128),denom:"ucosm".to_string()},
            desc:Some("Very Nice Lap-Top".to_string()),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(_res.attributes.len(),2);
        assert_eq!(_res.attributes[0],attr("method","add_new"));


         // query
         let res = query(deps.as_ref(), mock_env(), QueryMsg::QueryListing {id:1}).unwrap();
         let value: ListingResponse = from_binary(&res).unwrap();
         assert_eq!("lap-top", value.listing.name);


         // Success update
         let info = mock_info("user-one", &coins(10000, "ucosm"));
         let msg = ExecuteMsg::UpdateListing {
             id:1,
             price:Some(Coin{amount:Uint128::from(200 as u128),denom:"ucosm".to_string()}),
             name:Some("tv".to_string()),
             desc:Option::None,
         };
         let _res = execute(deps.as_mut(), mock_env(), info, msg);
         match _res {
            Err(ContractError::NameTooShort{length, min_length}) => {}
            _ => panic!("Name too short"),
        }

         // Success update
         let info = mock_info("user-one", &coins(10000, "ucosm"));
         let msg = ExecuteMsg::UpdateListing {
             id:1,
             price:Some(Coin{amount:Uint128::from(200 as u128),denom:"ucosm".to_string()}),
             name:Some("lcd-tv".to_string()),
             desc:Option::None,
         };
         let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
         assert_eq!(_res.attributes.len(),2);
         assert_eq!(_res.attributes[0],attr("method","update"));


         // query
        let res = query(deps.as_ref(), mock_env(), QueryMsg::QueryListing {id:1}).unwrap();
        let value: ListingResponse = from_binary(&res).unwrap();
        assert_eq!("lcd-tv", value.listing.name);


        // Invalid Item Name - Fail
        let info = mock_info("user-one", &coins(10000, "ucosm"));
        let msg = ExecuteMsg::Register {user:String::from("User1")};
        let _res = execute(deps.as_mut(), mock_env(), info, msg);
        match _res {
            Err(ContractError::InvalidCharacter{c}) => {}
            _ => panic!("Invalid name Character"),
        }
        
        
    }
    


    #[test]
    fn buy() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {storeName:String::from("village1"),refundPeriodPolicy:Some(50)};
        let info = mock_info("creator", &coins(10000, "ucosm"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        
        // Register user
        let info = mock_info("user-one", &coins(10000, "ucosm"));
        let msg = ExecuteMsg::Register {user:String::from("user1")};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(_res.attributes.len(),2);
        assert_eq!(_res.attributes[0],attr("method","register"));

        // Success create
        let info = mock_info("user-one", &coins(10000, "ucosm"));
        let msg = ExecuteMsg::ListItem {
            name:"lap-top".to_string(),
            price:Coin{amount:Uint128::from(150 as u128),denom:"ucosm".to_string()},
            desc:Some("Very Nice Lap-Top".to_string()),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(_res.attributes.len(),2);
        assert_eq!(_res.attributes[0],attr("method","add_new"));


         // query
         let res = query(deps.as_ref(), mock_env(), QueryMsg::QueryListing {id:1}).unwrap();
         let value: ListingResponse = from_binary(&res).unwrap();
         assert_eq!("lap-top", value.listing.name);


        // Buy failed - Buy from self
        let info = mock_info("user-one", &coins(10000, "ucosm"));
        let msg = ExecuteMsg::Buy {id:1};
        let _res = execute(deps.as_mut(), mock_env(), info, msg);
        match _res {
            Err(ContractError::BuyFromSelf{}) => {}
            _ => panic!("Cannot buy from self"),
        }

        // Buy failed - not enough funds
        let info = mock_info("user-two", &coins(15, "ucosm"));
        let msg = ExecuteMsg::Buy {id:1};
        let _res = execute(deps.as_mut(), mock_env(), info, msg);
        match _res {
            Err(ContractError::InsufficientFunds{exp,rec}) => {}
            _ => panic!("insufficient funds"),
        }

        // Buy item
        let info = mock_info("user-two", &coins(10000, "ucosm"));
        let msg = ExecuteMsg::Buy {id:1};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(_res.attributes.len(),3);
        assert_eq!(_res.attributes[0],attr("method","buy"));
        assert_eq!(_res.attributes[2],attr("buyer","user-two"));


        // Buy failed - Already bought
        let info = mock_info("user-two", &coins(10000, "ucosm"));
        let msg = ExecuteMsg::Buy {id:1};
        let _res = execute(deps.as_mut(), mock_env(), info, msg);
        match _res {
            Err(value) => {}
            _ => panic!("Item was bought"),
        }

        
    }

    #[test]
    fn rate() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {storeName:String::from("village1"),refundPeriodPolicy:Some(50)};
        let info = mock_info("creator", &coins(10000, "ucosm"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Rate Fail - Already Rated
        let info = mock_info("user-one", &coins(10000, "ucosm"));
        let msg = ExecuteMsg::RateStore {rating:7};
        let _res = execute(deps.as_mut(), mock_env(), info, msg);
        match _res {
           Err(ContractError::GenericMsgErr{msg}) => {}
           _ => panic!("FAiled calculating rating. Rating above 5"),
       }

        // Rate Store
         let info = mock_info("user-one", &coins(10000, "ucosm"));
         let msg = ExecuteMsg::RateStore {rating:5};
         let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
         assert_eq!(_res.attributes.len(),2);
         assert_eq!(_res.attributes[0],attr("method","rate_store"));
         assert_eq!(_res.attributes[1],attr("rater","user-one"));

         // Rate Fail - Already Rated
         let info = mock_info("user-one", &coins(10000, "ucosm"));
         let msg = ExecuteMsg::RateStore {rating:5};
         let _res = execute(deps.as_mut(), mock_env(), info, msg);
         match _res {
            Err(ContractError::HasAlreadyRated{addr}) => {}
            _ => panic!("Already Rated"),
        }

        // Rate Store
        let info = mock_info("user-two", &coins(10000, "ucosm"));
        let msg = ExecuteMsg::RateStore {rating:5};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(_res.attributes.len(),2);
        assert_eq!(_res.attributes[0],attr("method","rate_store"));
        assert_eq!(_res.attributes[1],attr("rater","user-two"));

         // query
         let res = query(deps.as_ref(), mock_env(), QueryMsg::QueryStoreInfo {}).unwrap();
         let value: StoreInfoResponse = from_binary(&res).unwrap();
         assert_eq!("5.00", value.store_info.storeRating.rating);

          // Rate Store
        let info = mock_info("user-three", &coins(10000, "ucosm"));
        let msg = ExecuteMsg::RateStore {rating:3};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(_res.attributes.len(),2);
        assert_eq!(_res.attributes[0],attr("method","rate_store"));
        assert_eq!(_res.attributes[1],attr("rater","user-three"));

         // Rate Store
         let info = mock_info("user-four", &coins(10000, "ucosm"));
         let msg = ExecuteMsg::RateStore {rating:2};
         let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
         assert_eq!(_res.attributes.len(),2);
         assert_eq!(_res.attributes[0],attr("method","rate_store"));
         assert_eq!(_res.attributes[1],attr("rater","user-four"));


          // query
          let res = query(deps.as_ref(), mock_env(), QueryMsg::QueryStoreInfo {}).unwrap();
          let value: StoreInfoResponse = from_binary(&res).unwrap();
          assert_eq!("3.75", value.store_info.storeRating.rating);
          assert_eq!("4.00", value.store_info.storeRating.votes);
 
    }

    

}