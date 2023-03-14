use anyhow::{Context, Result};

use chrono::prelude::*;

use rust_decimal::Decimal;
use rusty_money::{iso, Money};

use yahoo_finance_api::YahooConnector;
use yahoo_finance_api::*;

static PROVIDER: once_cell::sync::Lazy<YahooConnector> =
    once_cell::sync::Lazy::new(|| YahooConnector::new());

macro_rules! stat_fmt {
    ($stat:expr) => {
        $stat["fmt"]
            .as_str()
            .unwrap_or("N/A")
            .to_string()
            .to_uppercase()
    };
}

pub struct MainInfo {
    pub last_quote: String,
    pub name: String,
    pub instrument_type: String,
    pub currency: String,
    pub chart: Vec<YQuoteBlock>,
    pub bankruptcy: bool,
}

pub struct ExtendedInfo {
    pub exchange_name: String,
    pub day_range: String,
    pub market_change: String,
    pub market_change_percent: String,
}

pub struct StatsInfo {
    pub open: String,
    pub high: String,
    pub low: String,
    pub volume: String,
    pub pe_ratio: String,
    pub market_cap: String,
    pub dividend_yield: String,
    pub beta: String,
    pub eps: String,
}

impl ExtendedInfo {
    pub fn market_change_neg(&self) -> bool {
        self.market_change.starts_with('-')
    }
}

pub fn stox_search_symbol(symbol: &str) -> Result<Vec<YQuoteItem>, anyhow::Error> {
    Ok(PROVIDER.search_ticker(&urlencoding::encode(symbol))?.quotes)
}

pub fn stox_get_main_info(symbol: &str) -> Result<MainInfo> {
    let latest_quotes = PROVIDER.get_latest_quotes(&urlencoding::encode(symbol), "1h")?;

    let last_quote = latest_quotes.last_quote()?.close;
    let last_quote = (last_quote * 100.0).round() as i64;
    let last_quote = Decimal::new(last_quote, 2); // limit to two decimal places

    let quote_item = &PROVIDER.search_ticker(symbol)?.quotes[0];
    let mut name = &quote_item.long_name;
    if name.is_empty() {
        name = &quote_item.short_name;
    }

    let meta = &latest_quotes.chart.result[0].meta;
    let currency = meta.currency.to_uppercase();
    let instrument_type = meta.instrument_type.to_string();

    let mut main_info = MainInfo {
        last_quote: last_quote.to_string(),
        name: name.to_string(),
        instrument_type,
        currency: currency.clone(),
        chart: latest_quotes.chart.result,

        // Typically, if a company is undergoing bankruptcy they will
        // add "Q" to the end of their stock symbol in 5-chars length
        bankruptcy: symbol.ends_with('Q') && symbol.len() == 5,
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
        urlencoding::encode(symbol)
    );

    let data = reqwest::blocking::get(url)?.text()?;
    let data: serde_json::Value = serde_json::from_str(&data)?;

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

    let mut market_change_percent = format!(
        "{:.2}",
        quote["regularMarketChangePercent"]
            .as_f64()
            .context("expected market change percent")?
    ) + "%";
    if !market_change_percent.starts_with('-') {
        market_change_percent.insert(0, '+');
    }

    Ok(ExtendedInfo {
        exchange_name,
        day_range,
        market_change,
        market_change_percent,
    })
}

pub fn stox_get_stats_info(symbol: &str) -> Result<StatsInfo> {
    let url = format!(
        "{}{}{}",
        "https://query1.finance.yahoo.com/v11/finance/quoteSummary/",
        urlencoding::encode(symbol),
        "?modules=defaultKeyStatistics,summaryDetail"
    );

    let data = reqwest::blocking::get(url)?.text()?;
    let data: serde_json::Value = serde_json::from_str(&data)?;

    let result = &data["quoteSummary"]["result"][0];
    let summary_detail = &result["summaryDetail"];

    let open = stat_fmt!(summary_detail["open"]);
    let high = stat_fmt!(summary_detail["dayHigh"]);
    let low = stat_fmt!(summary_detail["dayLow"]);
    let volume = stat_fmt!(summary_detail["volume"]);
    let pe_ratio = stat_fmt!(summary_detail["trailingPE"]);
    let market_cap = stat_fmt!(summary_detail["marketCap"]);
    let dividend_yield = stat_fmt!(summary_detail["dividendYield"]);
    let beta = stat_fmt!(summary_detail["beta"]);
    let eps = stat_fmt!(result["defaultKeyStatistics"]["trailingEps"]);

    Ok(StatsInfo {
        open,
        high,
        low,
        volume,
        pe_ratio,
        market_cap,
        dividend_yield,
        beta,
        eps,
    })
}

pub fn stox_get_sidebar_info(symbol: &str) -> Result<(MainInfo, ExtendedInfo)> {
    Ok((stox_get_main_info(symbol)?, stox_get_extended_info(symbol)?))
}

pub fn stox_get_datagrid_info(symbol: &str) -> Result<(MainInfo, ExtendedInfo, StatsInfo)> {
    Ok((
        stox_get_main_info(symbol)?,
        stox_get_extended_info(symbol)?,
        // The stats are non-critical so if for some reason they fail to load
        // we can do without them
        stox_get_stats_info(symbol).unwrap_or(StatsInfo {
            open: "???".to_owned(),
            high: "???".to_owned(),
            low: "???".to_owned(),
            volume: "???".to_owned(),
            pe_ratio: "???".to_owned(),
            market_cap: "???".to_owned(),
            dividend_yield: "???".to_owned(),
            beta: "???".to_owned(),
            eps: "???".to_owned(),
        }),
    ))
}

pub fn stox_get_chart_x_axis(
    main_info: &MainInfo,
    range: &str,
) -> Result<Vec<String>, anyhow::Error> {
    let mut axis: Vec<String> = vec![];

    for index in &main_info.chart {
        for timestamp in &index.timestamp {
            // The x-axis should show different things depending on the range.
            // For example, in the span of one day, we should show the time
            // instead of the day.
            match range {
                "1d" => {
                    let mut hour = Utc
                        .timestamp_opt(*timestamp as i64, 0)
                        .single()
                        .context("expected timestamp")?
                        .hour()
                        .to_string();
                    hour.push_str(&(":".to_string() + "00")); // the hour is now our total time
                    axis.push(hour.to_string());
                }
                "5d" | "1wk" | "1mo" => {
                    let mut day = Utc
                        .timestamp_opt(*timestamp as i64, 0)
                        .single()
                        .context("expected timestamp")?
                        .day()
                        .to_string();
                    let month = Utc
                        .timestamp_opt(*timestamp as i64, 0)
                        .single()
                        .context("expected timestamp")?
                        .month()
                        .to_string();
                    day.push_str(&("/".to_string() + &month.to_string()));
                    axis.push(day.to_string());
                }
                "3mo" | "6mo" | "1y" | "2y" => {
                    let month = Utc
                        .timestamp_opt(*timestamp as i64, 0)
                        .single()
                        .context("expected timestamp")?
                        .month()
                        .to_string();
                    axis.push(month.to_string());
                }
                "5y" | "10y" | "ytd" | "max" => {
                    let year = Utc
                        .timestamp_opt(*timestamp as i64, 0)
                        .single()
                        .context("expected timestamp")?
                        .year()
                        .to_string();
                    axis.push(year.to_string());
                }
                &_ => {
                    // default, something wildly ridiculous
                    unimplemented!()
                }
            }
        }
    }

    axis.dedup(); // remove duplicates

    Ok(axis)
}

pub fn stox_get_chart_y_axis(extended_info: &ExtendedInfo) -> Result<Vec<f64>, anyhow::Error> {
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

pub fn stox_get_quotes(symbol: String, range: &str) -> Result<Vec<f64>, anyhow::Error> {
    let response = PROVIDER.get_quote_range(&symbol, "1m", range)?;

    let mut axis: Vec<f64> = vec![];

    for index_first in response.quotes().into_iter() {
        for index in index_first.iter() {
            let quote = index.close;

            axis.push(quote);
        }
    }

    Ok(axis)
}

pub fn stox_scale_quotes(quotes: &mut [f64], height: i32) -> Vec<f64> {
    let max = quotes.iter().max_by(|a, b| a.total_cmp(b)).unwrap();
    let min = quotes.iter().min_by(|a, b| a.total_cmp(b)).unwrap();
    let mut ret: Vec<f64> = vec![];
    for i in quotes.iter() {
        let mut x = (i - min) / (max - min) * ((height as f64) - 5.0);
        if x < 0.0 {
            // Sanity check: Don't push negative numbers under the graph
            x *= -1.0;
        }
        ret.push(x);
    }
    ret
}
