use chrono::prelude::*;
use yahoo_finance_api as yahoo;
use yahoo::YResponse;

pub fn stox_get_main_info(symbol: String) -> (String, String) {
    let provider = yahoo::YahooConnector::new();

    let latest_quote = provider.get_latest_quotes(symbol.as_str(), "1h").unwrap().last_quote().unwrap().close;
    let latest_quote = format!("{:.2}", latest_quote);  // limit to two decimal places

    let name = provider.search_ticker(&symbol).unwrap().quotes[0].short_name.to_string();

    return (name, latest_quote)
}

pub fn stox_get_chart_x_axis(response: YResponse) -> Vec<String> {
    let mut axis: Vec<String> = vec![];
    for index in response.chart.result.into_iter() {
        let range = index.meta.range;  // (1d, 60m, etc.) indicators
        for timestamp in index.timestamp {
            // The x-axis should show different things depending on the range.
            // For example, in the span of one day, we should show the time
            // instead of the day.
            match range.as_str() {
                "1d" => {
                    let mut hour = Utc.timestamp_opt(timestamp as i64, 0).unwrap().hour().to_string();
                    let min = Utc.timestamp_opt(timestamp as i64, 0).unwrap().minute().to_string();
                    hour.push_str(&(":".to_string() + &min.to_string()));  // the hour is now our total time
                    axis.push(hour.to_string());
                },
                "5d" | "1wk" | "1mo" => {
                    let mut day = Utc.timestamp_opt(timestamp as i64, 0).unwrap().day().to_string();
                    let month = Utc.timestamp_opt(timestamp as i64, 0).unwrap().month().to_string();
                    day.push_str(&("/".to_string() + &month.to_string()));
                    axis.push(day.to_string());
                },
                "3mo" | "6mo" | "1y" | "2y" => {
                    let month = Utc.timestamp_opt(timestamp as i64, 0).unwrap().month().to_string();
                    axis.push(month.to_string());
                },
                "5y" | "10y" | "ytd" | "max" => {
                    let year =  Utc.timestamp_opt(timestamp as i64, 0).unwrap().year().to_string();
                    axis.push(year.to_string());
                }
                &_ => {  // default, something wildly ridiculous
                    // TODO: Do something!
                }
            }
        }
    };
    return axis;
}