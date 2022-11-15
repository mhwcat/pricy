use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Configuration {
    pub email: Option<EmailConfiguration>,
    pub products: Vec<ProductConfiguration>,
}

#[derive(Deserialize, Debug)]
pub struct ProductConfiguration {
    pub url: String,
    pub selector: String,
    pub use_selector_attr: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct EmailConfiguration {
    pub sender: String,
    pub recipients: Vec<String>,
    pub smtp_host: String,
    pub smtp_username: String,
    pub smtp_password: String,
}
