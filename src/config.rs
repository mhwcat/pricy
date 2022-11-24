use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Configuration {
    pub email: Option<EmailConfiguration>,
    pub products: Vec<ProductConfiguration>,
}

impl Configuration {
    pub fn is_product_notify_only_drop(&self, url: &str) -> bool {
        for p in &self.products {
            if p.url.eq_ignore_ascii_case(url) {
                return p.notify_only_drop.unwrap_or(false);
            }
        }

        false
    }
}

#[derive(Deserialize, Debug)]
pub struct ProductConfiguration {
    pub url: String,
    pub selector: String,
    pub use_selector_attr: Option<String>,
    pub notify_only_drop: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct EmailConfiguration {
    pub sender: String,
    pub recipients: Vec<String>,
    pub smtp_host: String,
    pub smtp_username: String,
    pub smtp_password: String,
}
