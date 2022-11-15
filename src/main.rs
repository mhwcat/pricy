use crate::error::PricyError;
use clap::{command, Parser};
use config::Configuration;
use error::PricyResult;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};
use time::{
    format_description::{self},
    OffsetDateTime,
};

mod config;
#[cfg(feature = "email")]
mod email;
mod error;

const USERAGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:106.0) Gecko/20100101 Firefox/106.0";

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "tool for tracking prices from various online stores",
    help_template = "
{before-help}{name} v.{version} by {author}
{about-with-newline}
{usage-heading} {usage}
{all-args}{after-help}"
)]
struct Args {
    /// path to database file  (e.g. "db.ron")
    #[arg(short, long)]
    database: std::path::PathBuf,

    /// path to products configuration file (e.g. "config.toml")
    #[arg(short, long)]
    config: std::path::PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct Product {
    url: String,
    price: f32,
    last_check_time: OffsetDateTime,
}

#[derive(Debug, Deserialize, Serialize)]
struct ProductDb {
    products: HashMap<String, Product>,
}

#[tokio::main]
async fn main() -> PricyResult<()> {
    let args = Args::parse();

    let date_format =
        format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second] UTC")?;

    let mut file_db = read_db(&args.database)?;

    let configuration = parse_toml(&args.config)?;

    let client = reqwest::Client::builder().user_agent(USERAGENT).build()?;

    let results: Vec<PricyResult<Product>> = futures::stream::iter(configuration.products)
        .map(|prod| {
            let client = &client;

            async move {
                println!("Fetching {}", prod.url);

                let price_str =
                    read_price_from_url(client, &prod.url, prod.use_selector_attr, &prod.selector)
                        .await;
                if let Ok(price_str) = price_str {
                    let price_str = sanitize(&price_str);
                    if let Ok(price_num) = price_str.parse() {
                        Ok(Product {
                            url: prod.url,
                            price: price_num,
                            last_check_time: OffsetDateTime::now_utc(),
                        })
                    } else {
                        Err(PricyError {
                            msg: format!("Failed parsing price for {}", prod.url),
                        })
                    }
                } else {
                    Err(PricyError {
                        msg: format!("Failed fetching price for {}", prod.url),
                    })
                }
            }
        })
        .buffer_unordered(32)
        .collect()
        .await;

    for result in results {
        match result {
            Ok(prod) => {
                if let Some(db_prod) = file_db.products.get(&prod.url) {
                    if !prod.price.eq(&db_prod.price) {
                        println!(
                            "Updating price for product {}: {:.2} -> {:.2} (last check at {})",
                            prod.url,
                            db_prod.price,
                            prod.price,
                            db_prod.last_check_time.format(&date_format)?
                        );

                        #[cfg(feature = "email")]
                        email::send_price_update_email_notification(
                            &prod.url,
                            db_prod.price,
                            prod.price,
                            &OffsetDateTime::now_utc().format(&date_format)?,
                            &configuration.email,
                        )?;
                    }
                } else {
                    println!("Adding product {} with price {}", prod.url, prod.price);
                }

                file_db.products.insert(prod.url.clone(), prod);
            }
            Err(err) => {
                println!("Failed fetching price: {}", err);
            }
        }
    }

    save_db(&file_db, &args.database)?;

    Ok(())
}

fn sanitize(s: &str) -> String {
    s.chars()
        .filter(|ch| [',', '.', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0'].contains(ch))
        .map(|ch| match ch {
            ',' => '.',
            _ => ch,
        })
        .collect()
}

fn read_db(path: &Path) -> PricyResult<ProductDb> {
    if !path.exists() {
        println!(
            "Database file does not exist, creating one in {}",
            path.display()
        );

        let empty_db = ProductDb {
            products: HashMap::new(),
        };
        save_db(&empty_db, path)?;
    }

    Ok(ron::from_str(&std::fs::read_to_string(path)?)?)
}

fn save_db(db: &ProductDb, path: &Path) -> PricyResult<()> {
    Ok(std::fs::write(path, ron::to_string(&db)?)?)
}

fn parse_toml(path: &Path) -> PricyResult<Configuration> {
    let file_content = std::fs::read_to_string(path)?;
    let config: Configuration = toml::from_str(&file_content)?;

    Ok(config)
}

async fn read_price_from_url(
    client: &reqwest::Client,
    url: &str,
    attr_name: Option<String>,
    selector: &str,
) -> PricyResult<String> {
    let body = client.get(url).send().await?.text().await?;

    let html_parsed = scraper::html::Html::parse_document(&body);
    let selector = scraper::selector::Selector::parse(selector).map_err(|_| PricyError {
        msg: "Parser error".to_string(),
    })?;
    let price_selector = html_parsed.select(&selector).next().ok_or(PricyError {
        msg: "Price element not found".to_string(),
    })?;

    if let Some(attr_name) = attr_name {
        Ok(price_selector
            .value()
            .attr(&attr_name)
            .ok_or(PricyError {
                msg: "Price attribute not found".to_string(),
            })?
            .to_string())
    } else {
        Ok(price_selector.inner_html())
    }
}
