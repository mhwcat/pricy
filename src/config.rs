use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Configuration {
    pub email: Option<EmailConfiguration>,
    pub products: Vec<ProductConfiguration>,
}

impl Configuration {
    pub fn get_product_configuration(&self, url: &str) -> Option<&ProductConfiguration> {
        for p in &self.products {
            if p.url.eq_ignore_ascii_case(url) {
                return Some(p);
            }
        }

        None
    }
}

#[derive(Deserialize, Debug)]
pub struct ProductConfiguration {
    pub url: String,
    pub selector: String,
    pub use_selector_attr: Option<String>,
    pub notify_only_drop: Option<bool>,
    pub notification_email_recipients: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
pub struct EmailConfiguration {
    pub sender: String,
    pub recipients: Vec<String>,
    pub smtp_host: String,
    pub smtp_username: String,
    pub smtp_password: String,
}
