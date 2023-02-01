use chrono::prelude::*;
use rust_decimal::Decimal;
use rusty_money::{iso, Money};
use yahoo::{YResponse, YahooConnector};
use yahoo_finance_api as yahoo;

pub fn stox_get_main_info(provider: &YahooConnector, symbol: &str) -> (String, String) {
    let latest_quotes = provider.get_latest_quotes(symbol, "1h").unwrap();

    let last_quote = latest_quotes.last_quote().unwrap().close;
    let last_quote = (last_quote * 100.0).trunc() as i64;
    let last_quote = Decimal::new(last_quote, 2); // limit to two decimal places

    let ref short_name = provider.search_ticker(&symbol).unwrap().quotes[0].short_name;

    let currency = &latest_quotes.chart.result[0].meta.currency.to_uppercase();

    let last_quote = Money::from_decimal(last_quote, iso::find(&currency).unwrap()).to_string();

    return (last_quote, short_name.clone());
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

pub fn stox_get_chart_x_axis(response: YResponse) -> Vec<String> {
    let mut axis: Vec<String> = vec![];
    for index in response.chart.result.into_iter() {
        let range = index.meta.range; // (1d, 60m, etc.) indicators
        for timestamp in index.timestamp {
            // The x-axis should show different things depending on the range.
            // For example, in the span of one day, we should show the time
            // instead of the day.
            match range.as_str() {
                "1d" => {
                    let mut hour = Utc
                        .timestamp_opt(timestamp as i64, 0)
                        .unwrap()
                        .hour()
                        .to_string();
                    let min = Utc
                        .timestamp_opt(timestamp as i64, 0)
                        .unwrap()
                        .minute()
                        .to_string();
                    hour.push_str(&(":".to_string() + &min.to_string())); // the hour is now our total time
                    axis.push(hour.to_string());
                }
                "5d" | "1wk" | "1mo" => {
                    let mut day = Utc
                        .timestamp_opt(timestamp as i64, 0)
                        .unwrap()
                        .day()
                        .to_string();
                    let month = Utc
                        .timestamp_opt(timestamp as i64, 0)
                        .unwrap()
                        .month()
                        .to_string();
                    day.push_str(&("/".to_string() + &month.to_string()));
                    axis.push(day.to_string());
                }
                "3mo" | "6mo" | "1y" | "2y" => {
                    let month = Utc
                        .timestamp_opt(timestamp as i64, 0)
                        .unwrap()
                        .month()
                        .to_string();
                    axis.push(month.to_string());
                }
                "5y" | "10y" | "ytd" | "max" => {
                    let year = Utc
                        .timestamp_opt(timestamp as i64, 0)
                        .unwrap()
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
    return axis;
}
