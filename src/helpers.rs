use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cw_storage_plus::{Item, Map};
use cosmwasm_std::Coin;
use cosmwasm_std::{Uint128};
use rust_decimal::prelude::*;       
use crate::state::{STORE, Rating};

use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, CustomQuery, Querier, QuerierWrapper, StdResult, WasmMsg, WasmQuery,Storage
};

use crate::error::ContractError;


const MIN_NAME_LENGTH: u64 = 3;
const MAX_NAME_LENGTH: u64 = 64;

// auto-incrementing ID
pub fn update_counter(store: &mut dyn Storage, counter: &Item<u64>) -> StdResult<u64>{
    let id: u64 = (counter.may_load(store))?.unwrap_or_default() + 1;
    counter.save(store, &id)?;
    Ok(id)
}

// Check for valid amount and denom
pub fn check_for_valid_coin(
    sent:&Coin,
    required: String
) -> Result <(), ContractError> {
    if sent.denom != required {
      Err(ContractError::InvalidDenom{exp:required,rec:sent.denom.clone()}
    )
    } else {
        return Ok(());
    }
}

// Calculate average rating
pub fn calculate_rating(
    store: &mut dyn Storage,
    newRating:u16,
    curRating:&Rating
) -> Result <Rating, Box<dyn std::error::Error>> { 
       if newRating > 5 || newRating < 1{
        return Err(Box::new(ContractError::InvalidRating{}))
       }else{

        let oldAvg = Decimal::from_str(&curRating.rating)?;
        let oldVotesCount = Decimal::from_str(&curRating.votes)?;
        let newVotesCount = oldVotesCount + Decimal::from_str("1.00")?;
        let convertedRating = newRating.to_string() + ".00";
        let innerCalc = oldVotesCount / newVotesCount;
        innerCalc.to_string();

        
        let mut calculatedRating = (oldAvg * innerCalc) + 
        (Decimal::from_str(&convertedRating)? / newVotesCount); 
        
        let updated = Rating{
            rating:calculatedRating.round_dp(2).to_string(),
            votes:newVotesCount.to_string()

        };
        
        return Ok(updated);

       }

    
}

// let's not import a regexp library and just do these checks by hand
fn invalid_char(c: char) -> bool {
    let is_valid = c.is_digit(10) || c.is_ascii_lowercase() || (c == '.' || c == '-' || c == '_');
    !is_valid
}

pub fn validate_name (name: &str) -> Result<(),ContractError> {
    let length = name.len() as u64;
    if(name.len() as u64) < MIN_NAME_LENGTH {
        Err(ContractError::NameTooShort {
            length,
            min_length: MIN_NAME_LENGTH,
        })
    } else if (name.len() as u64) > MAX_NAME_LENGTH {
        Err(ContractError::NameTooLong {
            length,
            max_length: MAX_NAME_LENGTH,
        })
    }else {
        match name.find(invalid_char) {
            None => Ok(()),
            Some(bytepos_invalid_char_start) => {
                let c = name[bytepos_invalid_char_start..].chars().next().unwrap();
                Err(ContractError::InvalidCharacter { c })
            }
        }
    }
}
