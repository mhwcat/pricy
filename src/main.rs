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
    title: String,
    url: String,
    price: f32,
    last_check_time: OffsetDateTime,
}

#[derive(Debug, Deserialize, Serialize)]
struct ProductDb {
    products: HashMap<String, Product>,
}

#[derive(Debug)]
struct SiteData {
    title: String,
    price: String,
}

#[tokio::main]
async fn main() -> PricyResult<()> {
    let args = Args::parse();

    let date_format =
        format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second] UTC")?;

    let mut file_db = read_db(&args.database)?;

    let configuration = parse_toml(&args.config)?;
    let products = &configuration.products;

    let client = reqwest::Client::builder().user_agent(USERAGENT).build()?;

    let results: Vec<PricyResult<Product>> = futures::stream::iter(products)
        .map(|prod| {
            let client = &client;

            async move {
                println!("Fetching {}", prod.url);

                let site_data = read_data_from_url(
                    client,
                    &prod.url,
                    prod.use_selector_attr.clone(),
                    &prod.selector,
                )
                .await;
                match site_data {
                    Ok(site_data) => {
                        let price_str = sanitize(&site_data.price);
                        if let Ok(price_num) = price_str.parse() {
                            Ok(Product {
                                title: site_data.title,
                                url: prod.url.clone(),
                                price: price_num,
                                last_check_time: OffsetDateTime::now_utc(),
                            })
                        } else {
                            Err(PricyError {
                                msg: format!("Failed parsing price for {}", prod.url),
                            })
                        }
                    }
                    Err(err) => Err(PricyError {
                        msg: format!("Failed fetching price for {} with error {}", prod.url, err),
                    }),
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
                    if !db_prod.price.eq(&prod.price) {
                        println!(
                            "Updating price for product \"{}\": {:.2} -> {:.2} (last check at {})",
                            prod.title,
                            db_prod.price,
                            prod.price,
                            db_prod.last_check_time.format(&date_format)?
                        );

                        #[cfg(feature = "email")]
                        {
                            let notify_only_drop =
                                configuration.is_product_notify_only_drop(&prod.url);

                            if !notify_only_drop || prod.price < db_prod.price {
                                email::send_price_update_email_notification(
                                    &prod.url,
                                    &prod.title,
                                    db_prod.price,
                                    prod.price,
                                    &OffsetDateTime::now_utc().format(&date_format)?,
                                    &configuration.email,
                                )?;
                            }
                        }
                    }
                } else {
                    println!(
                        "Adding product \"{}\" with price {}",
                        prod.title, prod.price
                    );
                }

                file_db.products.insert(prod.url.clone(), prod);
            }
            Err(err) => {
                println!("Error: {}", err);
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

async fn read_data_from_url(
    client: &reqwest::Client,
    url: &str,
    attr_name: Option<String>,
    selector: &str,
) -> PricyResult<SiteData> {
    let body = client.get(url).send().await?.text().await?;

    let html_parsed = scraper::html::Html::parse_document(&body);
    let title_selector = scraper::selector::Selector::parse("title").map_err(|_| PricyError {
        msg: "Parser error".to_string(),
    })?;
    let price_selector = scraper::selector::Selector::parse(selector).map_err(|_| PricyError {
        msg: "Parser error".to_string(),
    })?;
    let title_element = html_parsed
        .select(&title_selector)
        .next()
        .ok_or(PricyError {
            msg: "Title element not found".to_string(),
        })?;
    let price_element = html_parsed
        .select(&price_selector)
        .next()
        .ok_or(PricyError {
            msg: "Price element not found".to_string(),
        })?;

    let price = if let Some(attr_name) = attr_name {
        price_element
            .value()
            .attr(&attr_name)
            .ok_or(PricyError {
                msg: "Price attribute not found".to_string(),
            })?
            .to_string()
    } else {
        price_element.inner_html()
    };

    Ok(SiteData {
        title: title_element.inner_html(),
        price,
    })
}
