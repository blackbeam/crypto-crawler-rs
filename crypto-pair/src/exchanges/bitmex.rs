use crypto_market_type::MarketType;

pub(crate) fn normalize_currency(currency: &str) -> String {
    if currency == "XBT" { "BTC" } else { currency }.to_string()
}

pub(crate) fn normalize_pair(mut symbol: &str) -> Option<String> {
    if symbol[(symbol.len() - 2)..].parse::<f64>().is_ok() {
        symbol = &symbol[..(symbol.len() - 3)]
    }

    let (base, quote) = if symbol.ends_with("USD") {
        (
            symbol.strip_suffix("USD").unwrap().to_string(),
            "USD".to_string(),
        )
    } else if symbol.ends_with("USDT") {
        (
            symbol.strip_suffix("USDT").unwrap().to_string(),
            "USDT".to_string(),
        )
    } else if symbol.ends_with("EUR") {
        (
            symbol.strip_suffix("EUR").unwrap().to_string(),
            "EUR".to_string(),
        )
    } else {
        let base_symbol = symbol;
        let quote_symbol = if base_symbol == "XBT" { "USD" } else { "XBT" };
        (base_symbol.to_string(), quote_symbol.to_string())
    };

    Some(format!(
        "{}/{}",
        normalize_currency(&base),
        normalize_currency(&quote)
    ))
}

pub(crate) fn get_market_type(symbol: &str) -> MarketType {
    let is_future = {
        let date = &symbol[(symbol.len() - 2)..];
        date.parse::<i64>().is_ok()
    };
    let real_symbol = if is_future {
        &symbol[..(symbol.len() - 3)]
    } else {
        symbol
    };
    // 0, linear; 1, inverse; 2, quanto
    let linear_inverse_quanto = if real_symbol.ends_with("USDT") {
        0
    } else if real_symbol.starts_with("XBT") {
        1
    } else if real_symbol.ends_with("USD") || real_symbol.ends_with("EUR") {
        2
    } else {
        // Settled in XBT, quoted in XBT
        debug_assert_eq!(symbol.len(), 6);
        0
    };

    match linear_inverse_quanto {
        0 => {
            if is_future {
                MarketType::LinearFuture
            } else {
                MarketType::LinearSwap
            }
        }
        1 => {
            if is_future {
                MarketType::InverseFuture
            } else {
                MarketType::InverseSwap
            }
        }
        2 => {
            if is_future {
                MarketType::QuantoFuture
            } else {
                MarketType::QuantoSwap
            }
        }
        _ => panic!("Impossible {}", symbol),
    }
}
