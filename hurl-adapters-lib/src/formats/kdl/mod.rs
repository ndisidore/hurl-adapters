//! KDL format translator for converting KDL documents to Hurl format.
//!
//! # Example
//!
//! ```
//! use kdl::KdlDocument;
//! use hurl_adapters_lib::formats::kdl::{translate, translate_to_string};
//!
//! let kdl: KdlDocument = r#"
//! GET "https://example.com" {
//!     expect {
//!         status 200
//!     }
//! }
//! "#.parse().unwrap();
//!
//! // Get hurl file AST
//! let hurl_file = translate(&kdl).unwrap();
//!
//! // Or get hurl string directly
//! let hurl_string = translate_to_string(&kdl).unwrap();
//! ```

mod body;
pub mod error;
mod request;
mod response;
mod translator;

pub use error::{Result, TranslationError};
pub use translator::{translate, translate_to_string};

/// Valid HTTP methods supported for KDL request nodes.
pub const VALID_HTTP_METHODS: &[&str] = &[
    "GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS", "CONNECT", "TRACE",
];
