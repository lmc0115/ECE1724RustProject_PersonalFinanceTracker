// exchange_scraper.rs
// Website: https://www.x-rates.com/table/?from=CAD&amount=1

// Functions:
// 1. Scrape the FX rates and the date on website.
// 2. Check if needs to scrape or not automatically
//   1) if db contains the date's FX rates already, no need to scrape.
//.  2) if db doesn't contain the date's FX rates, scrape and insert.

use chrono::{DateTime, NaiveDate, Utc};
use reqwest::Client;
use scraper::{Html, Selector};
use sqlx::SqlitePool;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ExchangeRate {
    pub from_currency: String,
    pub to_currency: String,
    pub rate: f64,
    pub rate_date: NaiveDate,
}

pub struct ExchangeRateScraper {
    client: Client,
    base_url: String,
}

impl ExchangeRateScraper {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap();

        Self {
            client,
            base_url: "https://www.x-rates.com/table/".to_string(),
        }
    }

    // check if db contains the date's FX rates.
    pub async fn check_if_up_to_date(
        &self,
        pool: &SqlitePool,
        from_currency: &str,
        date: NaiveDate,
    ) -> Result<bool, sqlx::Error> {
        let date_str = date.format("%Y-%m-%d").to_string();

        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM exchange_rates
            WHERE from_currency = ?
            AND DATE(rate_date) = ?
            AND source = 'scraper'
            "#,
        )
        .bind(from_currency)
        .bind(date_str)
        .fetch_one(pool)
        .await?;

        Ok(count > 0)
    }

    // Only scrape if db doesn't contain the date's FX rates
    pub async fn smart_fetch_exchange_rates(
        &self,
        pool: &SqlitePool,
        from_currency: &str,
    ) -> Result<(Vec<ExchangeRate>, bool), Box<dyn std::error::Error>> {
        println!(
            "1. Checking if {}'s FX rates need to be updated...",
            from_currency
        );

        let url = format!("{}?from={}&amount=1", self.base_url, from_currency);
        let response = self.client.get(&url).send().await?;
        if !response.status().is_success() {
            return Err(format!("HTTP Error: {}", response.status()).into());
        }

        let html = response.text().await?;
        let rate_date = self.extract_timestamp(&html)?;

        let is_up_to_date = self
            .check_if_up_to_date(pool, from_currency, rate_date)
            .await?;

        if is_up_to_date {
            println!(
                "  1.3 DB contains the FX rates of {} already，no need to update.",
                rate_date
            );
            return Ok((Vec::new(), true));
        }

        println!(
            "  1.3 DB doesn't contain the FX rates of {} yet, scrape {}.",
            rate_date, url
        );

        let rates = self.parse_exchange_rates(&html, from_currency, rate_date)?;
        Ok((rates, false))
    }

    fn extract_timestamp(&self, html: &str) -> Result<NaiveDate, Box<dyn std::error::Error>> {
        let document = Html::parse_document(html);
        let timestamp_selector = Selector::parse(".ratesTimestamp").unwrap();

        if let Some(timestamp_element) = document.select(&timestamp_selector).next() {
            let timestamp_text = timestamp_element.text().collect::<String>();

            let date_str = self.parse_date_from_timestamp(&timestamp_text)?;

            println!("  1.1 Original timestamp online: {}", timestamp_text.trim());
            println!("  1.2 Online Date after parsing: {}", date_str);

            return Ok(date_str);
        }

        println!("  1.1 If original timestamp online not found, use current timestamp.");
        Ok(Utc::now().date_naive())
    }

    fn parse_date_from_timestamp(
        &self,
        timestamp: &str,
    ) -> Result<NaiveDate, Box<dyn std::error::Error>> {
        let timestamp = timestamp.trim();

        let formats = vec![
            "%b %d, %Y", // Dec 06, 2024
            "%B %d, %Y", // December 06, 2024
            "%b %e, %Y", // Dec 6, 2024
            "%B %e, %Y", // December 6, 2024
        ];

        let date_part = timestamp
            .split(" UTC")
            .next()
            .unwrap_or(timestamp)
            .split_whitespace()
            .take(3)
            .collect::<Vec<_>>()
            .join(" ");

        for format in formats {
            if let Ok(date) = NaiveDate::parse_from_str(&date_part, format) {
                return Ok(date);
            }
        }

        println!(
            "Cannot parse the timestamp: '{}' online, use current timestamp instead.",
            timestamp
        );
        Ok(Utc::now().date_naive())
    }

    fn parse_exchange_rates(
        &self,
        html: &str,
        from_currency: &str,
        rate_date: NaiveDate,
    ) -> Result<Vec<ExchangeRate>, Box<dyn std::error::Error>> {
        let document = Html::parse_document(html);
        let mut rates = Vec::new();

        let table_selector = Selector::parse("table.tablesorter").unwrap();
        let row_selector = Selector::parse("tbody tr").unwrap();
        let cell_selector = Selector::parse("td").unwrap();
        let link_selector = Selector::parse("a").unwrap();

        if let Some(table) = document.select(&table_selector).next() {
            for row in table.select(&row_selector) {
                let cells: Vec<_> = row.select(&cell_selector).collect();

                if cells.len() >= 2 {
                    let currency_name = cells[0].text().collect::<String>().trim().to_string();
                    let currency_code = if let Some(link) = cells[1].select(&link_selector).next() {
                        if let Some(href) = link.value().attr("href") {
                            self.extract_currency_code_from_url(href)
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    let to_currency = if let Some(code) = currency_code {
                        format!("{} ({})", currency_name, code)
                    } else {
                        currency_name
                    };

                    let rate_text = cells[1].text().collect::<String>().trim().to_string();

                    if let Ok(rate) = rate_text.replace(",", "").parse::<f64>() {
                        rates.push(ExchangeRate {
                            from_currency: from_currency.to_string(),
                            to_currency,
                            rate,
                            rate_date,
                        });
                    }
                }
            }
        }

        if rates.is_empty() {
            return Err("2. Cannot find foreign exchange rates data.".into());
        }

        println!("2. Parse {} exchange rates successfully.", rates.len());
        Ok(rates)
    }

    fn extract_currency_code_from_url(&self, url: &str) -> Option<String> {
        // find the currency code: the value of the "to" parameter
        if let Some(to_pos) = url.find("to=") {
            let start = to_pos + 3;
            let remaining = &url[start..];

            let end = remaining.find('&').unwrap_or(remaining.len());
            let code = &remaining[..end];

            if !code.is_empty() {
                return Some(code.to_string());
            }
        }
        None
    }

    pub async fn smart_fetch_multiple(
        &self,
        pool: &SqlitePool,
        currencies: Vec<&str>,
    ) -> HashMap<String, (Vec<ExchangeRate>, bool)> {
        let mut results = HashMap::new();

        for currency in currencies {
            println!("\nScraping {} exchange rates...", currency);

            match self.smart_fetch_exchange_rates(pool, currency).await {
                Ok((rates, was_up_to_date)) => {
                    if was_up_to_date {
                        println!("2. Latest FX Rates of {} are scraped already.", currency)
                    } else {
                        println!("3. ✓Success: get {} exchange rates.", rates.len());
                    }
                    results.insert(currency.to_string(), (rates, was_up_to_date));
                }
                Err(e) => {
                    eprintln!("3. ✗Error: {}.", e);
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }

        results
    }

    pub async fn save_to_database(
        &self,
        pool: &SqlitePool,
        rates: &[ExchangeRate],
    ) -> Result<usize, sqlx::Error> {
        let now = Utc::now();
        let mut saved_count = 0;

        for rate in rates {
            sqlx::query!(
                r#"
                INSERT INTO exchange_rates (from_currency, to_currency, rate, rate_date, source)
                VALUES (?, ?, ?, ?, 'scraper')
                "#,
                rate.from_currency,
                rate.to_currency,
                rate.rate,
                now
            )
            .execute(pool)
            .await?;

            saved_count += 1;
        }

        Ok(saved_count)
    }
}

pub fn print_exchange_rates(rates: &[ExchangeRate]) {
    if rates.is_empty() {
        println!("No data for foreign exchange rates.");
        return;
    }

    let from_currency = &rates[0].from_currency;

    println!("\n╔════════════════════════════════════════╗");
    println!("║  {} Foreign Exchange Rates             ║", from_currency);
    println!("╠════════════════════════════════════════╣");
    println!("║  Currency    │    Exchange Rates       ║");
    println!("╠════════════════════════════════════════╣");

    for rate in rates {
        println!(
            "║  {:>4}    │  {:<10.6}            ║",
            rate.to_currency, rate.rate
        );
    }

    println!("╚════════════════════════════════════════╝");
}
