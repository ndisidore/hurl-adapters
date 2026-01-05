//! Hurl Adapters Library
//!
//! This library provides translation from various formats to Hurl format.
//!
//! # Supported Formats
//!
//! - **KDL**: KDL Document Language support via `formats::kdl`
//!
//! # Example
//!
//! ```
//! use kdl::KdlDocument;
//! use hurl_adapters_lib::formats::kdl::{translate_to_string, TranslationError};
//!
//! let kdl: KdlDocument = r#"
//! GET "https://example.com" {
//!     expect {
//!         status 200
//!     }
//! }
//! "#.parse().unwrap();
//!
//! let hurl = translate_to_string(&kdl).unwrap();
//! assert!(hurl.contains("GET https://example.com"));
//! ```

pub mod formats;
pub mod writer;
