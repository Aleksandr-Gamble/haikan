//! Haikan (配管, meaning "plumbing")
//!
//! 配管 is libray to make the "plumbing" or connections easier in the Tokio asynchronous
//! ecosystem. It also contains some functionality to make common, simple tasks more ergonomic.
//!
//! NOTE: Several modules use NoTls by default- use within a private network may be acceptable but
//! be cautious regarding use over public networks without Tls security.
//!
//!
pub mod err;
pub mod opensearch;
pub mod postgres;
pub mod redis;
pub mod sqs;
pub mod clean_text;
pub mod utils;

