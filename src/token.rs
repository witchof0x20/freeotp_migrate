// Copyright 2018 witchof0x20
/*This file is part of freeotp_migrate.

    freeotp_migrate is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    freeotp_migrate is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with freeotp_migrate.  If not, see <http://www.gnu.org/licenses/>.
*/
// Used for implementing display traits
use std::fmt;
// Used for url encoding
use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};
// Used for base32 encoding
use base32::{Alphabet, encode as base32_encode};
// Used for qr code generation
#[cfg(feature = "qrcode_create")]
use qrcode::QrCode;
#[cfg(feature = "qrcode_create")]
use image::Luma;

/// Stores data about a 2FA token
#[derive(Serialize, Deserialize, Debug)]
pub struct Token<'a> {
    /// 2FA token type
    /// Valid types are `hotp` and `totp` to distinguish
    /// whether the key will be used for counter-based HOTP
    /// or for TOTP
    #[serde(rename = "type")]
    #[serde(default)]
    pub token_type: TokenType,
    /// Label (usually username)
    pub label: &'a str,
    /// Alternate label (usually exists if entry was created manually)
    #[serde(rename = "labelAlt")]
    pub label_alt: Option<&'a str>,
    /// Secret key
    pub secret: Vec<i8>,
    /// External issuer of token
    #[serde(rename = "issuerExt")]
    pub issuer_ext: Option<&'a str>,
    /// Internal issuer of 2fa token
    #[serde(rename = "issuerInt")]
    pub issuer_int: Option<&'a str>,
    /// Alternate issuer
    #[serde(rename = "issuerAlt")]
    pub issuer_alt: Option<&'a str>,
    /// What 2FA algorithm to use
    #[serde(rename = "algo")]
    #[serde(default)]
    pub algorithm: TokenAlgorithm,
    /// Number of digits the 2fa token should output
    #[serde(default = "default_num_digits")]
    pub digits: usize,
    /// Counter (only required for HOTP)
    #[serde(default = "default_counter")]
    pub counter: usize,
    /// Update frequency
    #[serde(default = "default_period")]
    pub period: usize,
}

/// 2FA token type
#[derive(Serialize, Deserialize, Debug)]
pub enum TokenType {
    TOTP,
    HOTP,
}

/// Set default value for TokenType
impl Default for TokenType {
    /// Returns the default value (TOTP)
    fn default() -> TokenType {
        TokenType::TOTP
    }
}

/// Allows the enum to be printed and converted to a string
impl fmt::Display for TokenType {
    /// Function to allows enum to be printed
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{}",
            match self {
                &TokenType::TOTP => "TOTP",
                &TokenType::HOTP => "HOTP",
            }
        )
    }
}

/// 2FA token algorithm
#[derive(Serialize, Deserialize, Debug)]
pub enum TokenAlgorithm {
    SHA1,
    SHA256,
    SHA512,
}

/// Set default value for TokenType
impl Default for TokenAlgorithm {
    /// Returns the default value (TOTP)
    fn default() -> TokenAlgorithm {
        TokenAlgorithm::SHA1
    }
}

/// Allows the enum to be printed and converted to a string
impl fmt::Display for TokenAlgorithm {
    /// Function to allows enum to be printed
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{}",
            match self {
                &TokenAlgorithm::SHA1 => "SHA1",
                &TokenAlgorithm::SHA256 => "SHA256",
                &TokenAlgorithm::SHA512 => "SHA512",
            }
        )
    }
}

/// Returns the default number of digits
/// 6 digits
fn default_num_digits() -> usize {
    6
}
/// Returns the default value for token update frequency
/// 30 seconds
fn default_period() -> usize {
    30
}

/// Returns the default counter value for HOTP
/// This should never happen unless algorithm is TOTP
fn default_counter() -> usize {
    0
}

/// Allows conversion to URI
impl<'de> fmt::Display for Token<'de> {
    /// Converts a token to a URI for use in a QR code
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        // Generate label
        let label: String = match self.issuer_ext {
            Some(issuer_ext) => format!("{}:{}", issuer_ext, self.label),
            None => self.label.to_string(),
        };
        // Generate issuer
        let issuer: String = match self.issuer_int {
            Some(issuer_int) => match self.issuer_ext {
                Some(issuer_ext) => format!("{}:{}", issuer_int, issuer_ext),
                None => issuer_int.to_string(),
            },
            None => match self.issuer_ext {
                Some(issuer_ext) => issuer_ext.to_string(),
                None => match self.issuer_alt {
                    Some(issuer_alt) => issuer_alt.to_string(),
                    None => match self.label_alt {
                        Some(label_alt) => label_alt.to_string(),
                        None => "Unknown label".to_string(),
                    },
                },
            },
        };
        // Generate the URL
        write!(
            formatter,
            "otpauth://{}/{}?secret={}&issuer={}&algorithm={}&digits={}&period={}",
            self.token_type.to_string().to_lowercase(),
            utf8_percent_encode(&label, DEFAULT_ENCODE_SET),
            base32_encode(
                Alphabet::RFC4648 { padding: true },
                &self.secret
                    .iter()
                    .map(|c| c.clone() as u8)
                    .collect::<Vec<u8>>()[..]
            ),
            utf8_percent_encode(&issuer, DEFAULT_ENCODE_SET),
            utf8_percent_encode(&self.algorithm.to_string(), DEFAULT_ENCODE_SET),
            self.digits,
            self.period
        )
    }
}
/// Allows conversion to qr
#[cfg(feature = "qrcode_create")]
impl<'de> Token<'de> {
    /// Returns a qr code object
    fn generate_qr(&self) -> QrCode {
        // Generates a qr code
        match QrCode::new(&self.to_string()) {
            Ok(qr_code) => qr_code,
            Err(error) => panic!("Error generating QR code: {}", error),
        }
    }
    /// Saves a qr code image
    pub fn save_qr(&self, output_filename: String) {
        match self.generate_qr()
            .render::<Luma<u8>>()
            .build()
            .save(output_filename)
        {
            Ok(_) => {}
            Err(error) => panic!("Error saving QR code: {}", error),
        }
    }
}
