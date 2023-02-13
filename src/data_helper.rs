use anyhow::{Context, Result};
use chrono::prelude::*;
use rust_decimal::Decimal;
use rusty_money::{iso, Money};
use serde_json::*;
use yahoo::YahooConnector;
use yahoo::*;
use yahoo_finance_api as yahoo;

pub struct MainInfo {
    pub last_quote: String,
    pub short_name: String,
    pub instrument_type: String,
    pub currency: String,
}

pub struct ExtendedInfo {
    pub exchange_name: String,
    pub day_range: String,
    pub market_change: String,
}

impl ExtendedInfo {
    pub fn market_change_neg(&self) -> bool {
        self.market_change.chars().nth(0).unwrap() == '-'
    }
}

pub fn stox_search_symbol(symbol: &str) -> Result<Vec<YQuoteItem>, anyhow::Error> {
    let provider = YahooConnector::new();
    Ok(provider.search_ticker(&urlencoding::encode(symbol))?.quotes)
}

pub fn stox_get_main_info(symbol: &str) -> Result<MainInfo> {
    let provider = yahoo::YahooConnector::new();
    let latest_quotes = provider.get_latest_quotes(symbol, "1h")?;

    let last_quote = latest_quotes.last_quote()?.close;
    let last_quote = (last_quote * 100.0).round() as i64;
    let last_quote = Decimal::new(last_quote, 2); // limit to two decimal places

    let ref short_name = provider.search_ticker(&symbol)?.quotes[0].short_name;

    let meta = &latest_quotes.chart.result[0].meta;
    let currency = meta.currency.to_uppercase();
    let instrument_type = (&meta.instrument_type).to_string();

    let mut main_info = MainInfo {
        last_quote: last_quote.to_string(),
        short_name: short_name.to_string(),
        instrument_type,
        currency: currency.clone(),
    };

    if let Some(currency) = iso::find(&currency) {
        let last_quote = Money::from_decimal(last_quote, currency).to_string();
        main_info.last_quote = last_quote;
    }

    Ok(main_info)
}

pub fn stox_get_extended_info(symbol: &str) -> Result<ExtendedInfo> {
    let url = format!(
        "https://query1.finance.yahoo.com/v7/finance/options/{}",
        symbol
    );

    let data = reqwest::blocking::get(url)?.text()?;
    let data: Value = serde_json::from_str(&data)?;

    let quote = &data["optionChain"]["result"][0]["quote"];

    let exchange_name = quote["fullExchangeName"]
        .as_str()
        .context("expected exchange name")?
        .to_owned();
    let day_range = quote["regularMarketDayRange"]
        .as_str()
        .context("expected day range")?
        .to_owned();

    let mut market_change = format!(
        "{:.2}",
        quote["regularMarketChange"]
            .as_f64()
            .context("expected market change")?
    );
    if !market_change.starts_with('-') {
        market_change.insert(0, '+');
    }

    Ok(ExtendedInfo {
        exchange_name,
        day_range,
        market_change,
    })
}

pub fn stox_get_complete_info(symbol: &str) -> Result<(MainInfo, ExtendedInfo)> {
    Ok((stox_get_main_info(symbol)?, stox_get_extended_info(symbol)?))
}

pub fn stox_get_ranges(symbol: String) -> Vec<String> {
    let provider = yahoo::YahooConnector::new();
    let valid_ranges = &provider
        .get_latest_quotes(&symbol, "1h")
        .unwrap()
        .chart
        .result[0]
        .meta
        .valid_ranges;
    valid_ranges.to_vec()
}

pub fn stox_get_chart_x_axis(symbol: String, range: &str) -> Result<Vec<String>, anyhow::Error> {
    let provider = YahooConnector::new();
    let response = provider.get_latest_quotes(&symbol, "1h")?;
    let mut axis: Vec<String> = vec![];
    for index in response.chart.result.into_iter() {
        for timestamp in index.timestamp {
            // The x-axis should show different things depending on the range.
            // For example, in the span of one day, we should show the time
            // instead of the day.
            match range {
                "1d" => {
                    let mut hour = Utc
                        .timestamp_opt(timestamp as i64, 0)
                        .single()
                        .context("expected timestamp")?
                        .hour()
                        .to_string();
                    hour.push_str(&(":".to_string() + "00")); // the hour is now our total time
                    axis.push(hour.to_string());
                }
                "5d" | "1wk" | "1mo" => {
                    let mut day = Utc
                        .timestamp_opt(timestamp as i64, 0)
                        .single()
                        .context("expected timestamp")?
                        .day()
                        .to_string();
                    let month = Utc
                        .timestamp_opt(timestamp as i64, 0)
                        .single()
                        .context("expected timestamp")?
                        .month()
                        .to_string();
                    day.push_str(&("/".to_string() + &month.to_string()));
                    axis.push(day.to_string());
                }
                "3mo" | "6mo" | "1y" | "2y" => {
                    let month = Utc
                        .timestamp_opt(timestamp as i64, 0)
                        .single()
                        .context("expected timestamp")?
                        .month()
                        .to_string();
                    axis.push(month.to_string());
                }
                "5y" | "10y" | "ytd" | "max" => {
                    let year = Utc
                        .timestamp_opt(timestamp as i64, 0)
                        .single()
                        .context("expected timestamp")?
                        .year()
                        .to_string();
                    axis.push(year.to_string());
                }
                &_ => { // default, something wildly ridiculous
                     // TODO: Do something!
                }
            }
        }
    }
    axis.dedup(); // remove duplicates
    return Ok(axis);
}

// Currently the yahoo finance api crate doesn't have support for getting market day ranges
pub fn stox_get_chart_y_axis(symbol: String) -> Result<Vec<f64>, anyhow::Error> {
    let extended_info = stox_get_extended_info(&symbol)?;

    // We have our range, but we need to make it a vec of points.
    let i = extended_info.day_range.trim();
    let i2: Vec<&str> = i.split(" - ").collect();
    let start: f64 = i2[0].parse()?;
    let end: f64 = i2[1].parse()?;
    let step_part1 = (start + end) / 2.0;
    let step = (step_part1 - start) / 2.0;

    Ok(vec![
        start,
        start + step,
        step_part1,
        step_part1 + step,
        end,
    ])
}

pub fn stox_get_quotes(symbol: String) -> Vec<f64> {
    let provider = YahooConnector::new();
    let response = provider.get_latest_quotes(&symbol, "30m").unwrap();
    let mut axis: Vec<f64> = vec![];
    for index_first in response.quotes().into_iter() {
        for index in index_first.iter() {
            let quote = index.close;

            axis.push(quote as f64);
        }
    }
    return axis;
}

pub fn stox_scale_quotes(quotes: &mut Vec<f64>, height: i32) -> Vec<f64> {
    let max = quotes.iter().max_by(|a, b| a.total_cmp(b)).unwrap();
    let min = quotes.iter().min_by(|a, b| a.total_cmp(b)).unwrap();
    let mut ret: Vec<f64> = vec![];
    for i in quotes.iter() {
        let mut x = (i - min) / (max - min) * height as f64;
        if x < 0.0 {
            // Don't push negative numbers under the graph
            x *= -1.0;
        }
        ret.push(x);
    }
    ret
}
