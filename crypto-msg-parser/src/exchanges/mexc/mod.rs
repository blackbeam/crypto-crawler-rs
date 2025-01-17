mod mexc_spot;
mod mexc_swap;

use std::collections::HashMap;

use crypto_market_type::MarketType;

use crate::{OrderBookMsg, TradeMsg};

use serde_json::Value;
use simple_error::SimpleError;

pub(super) const EXCHANGE_NAME: &str = "mexc";

pub(crate) fn extract_symbol(msg: &str) -> Result<String, SimpleError> {
    if let Ok(arr) = serde_json::from_str::<Vec<Value>>(msg) {
        Ok(arr[1]["symbol"].as_str().unwrap().to_string())
    } else if let Ok(json_obj) = serde_json::from_str::<HashMap<String, Value>>(msg) {
        Ok(json_obj
            .get("symbol")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string())
    } else {
        Err(SimpleError::new(format!(
            "Failed to extract symbol from {}",
            msg
        )))
    }
}

pub(crate) fn extract_timestamp(msg: &str) -> Result<Option<i64>, SimpleError> {
    if let Ok(arr) = serde_json::from_str::<Vec<Value>>(msg) {
        let channel = arr[0].as_str().unwrap();
        match channel {
            "push.symbol" => {
                let data = arr[1]["data"].as_object().unwrap();
                if let Some(deals) = data.get("deals") {
                    let raw_trades = deals.as_array().unwrap();
                    let timestamp = raw_trades.iter().fold(std::i64::MIN, |a, raw_trade| {
                        a.max(raw_trade["t"].as_i64().unwrap())
                    });

                    if timestamp == std::i64::MIN {
                        Err(SimpleError::new(format!("deals is empty in {}", msg)))
                    } else {
                        Ok(Some(timestamp))
                    }
                } else {
                    Ok(None)
                }
            }
            _ => Err(SimpleError::new(format!(
                "Unknown channel {} in {}",
                channel, msg
            ))),
        }
    } else if let Ok(json_obj) = serde_json::from_str::<HashMap<String, Value>>(msg) {
        if let Some(x) = json_obj.get("ts") {
            Ok(Some(x.as_i64().unwrap()))
        } else if let Some(deals) = json_obj["data"].get("deals") {
            let timestamp = deals
                .as_array()
                .unwrap()
                .iter()
                .fold(std::i64::MIN, |a, raw_trade| {
                    a.max(raw_trade["t"].as_i64().unwrap())
                });
            if timestamp == std::i64::MIN {
                Err(SimpleError::new(format!("deals is empty in {}", msg)))
            } else {
                Ok(Some(timestamp))
            }
        } else {
            Ok(None)
        }
    } else {
        Err(SimpleError::new(format!(
            "Failed to extract symbol from {}",
            msg
        )))
    }
}

pub(crate) fn parse_trade(
    market_type: MarketType,
    msg: &str,
) -> Result<Vec<TradeMsg>, SimpleError> {
    if market_type == MarketType::Spot {
        mexc_spot::parse_trade(msg)
    } else {
        mexc_swap::parse_trade(market_type, msg)
    }
}

pub(crate) fn parse_l2(
    market_type: MarketType,
    msg: &str,
    timestamp: Option<i64>,
) -> Result<Vec<OrderBookMsg>, SimpleError> {
    if market_type == MarketType::Spot {
        mexc_spot::parse_l2(
            msg,
            timestamp.expect("MEXC Spot orderbook messages don't have timestamp"),
        )
    } else {
        mexc_swap::parse_l2(market_type, msg)
    }
}
