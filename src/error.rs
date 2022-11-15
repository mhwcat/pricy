use std::fmt::Display;

#[derive(Debug)]
pub struct PricyError {
    pub msg: String,
}

pub type PricyResult<T> = Result<T, PricyError>;

impl From<std::io::Error> for PricyError {
    fn from(err: std::io::Error) -> Self {
        PricyError {
            msg: err.to_string(),
        }
    }
}

impl From<ron::Error> for PricyError {
    fn from(err: ron::Error) -> Self {
        PricyError {
            msg: err.to_string(),
        }
    }
}

impl From<ron::error::SpannedError> for PricyError {
    fn from(err: ron::error::SpannedError) -> Self {
        PricyError {
            msg: err.to_string(),
        }
    }
}

impl From<toml::de::Error> for PricyError {
    fn from(err: toml::de::Error) -> Self {
        PricyError {
            msg: err.to_string(),
        }
    }
}

impl From<reqwest::Error> for PricyError {
    fn from(err: reqwest::Error) -> Self {
        PricyError {
            msg: err.to_string(),
        }
    }
}

impl From<time::error::IndeterminateOffset> for PricyError {
    fn from(err: time::error::IndeterminateOffset) -> Self {
        PricyError {
            msg: err.to_string(),
        }
    }
}

impl From<time::error::InvalidFormatDescription> for PricyError {
    fn from(err: time::error::InvalidFormatDescription) -> Self {
        PricyError {
            msg: err.to_string(),
        }
    }
}

impl From<time::error::Format> for PricyError {
    fn from(err: time::error::Format) -> Self {
        PricyError {
            msg: err.to_string(),
        }
    }
}

#[cfg(feature = "email")]
impl From<lettre::transport::smtp::Error> for PricyError {
    fn from(err: lettre::transport::smtp::Error) -> Self {
        PricyError {
            msg: err.to_string(),
        }
    }
}

#[cfg(feature = "email")]
impl From<lettre::error::Error> for PricyError {
    fn from(err: lettre::error::Error) -> Self {
        PricyError {
            msg: err.to_string(),
        }
    }
}

#[cfg(feature = "email")]
impl From<lettre::address::AddressError> for PricyError {
    fn from(err: lettre::address::AddressError) -> Self {
        PricyError {
            msg: err.to_string(),
        }
    }
}

impl Display for PricyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.msg)?;

        Ok(())
    }
}
