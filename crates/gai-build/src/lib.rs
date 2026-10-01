//! The generic set builder: samples definitions into scenes, hands scenes to an
//! [`gai_core::export::Exporter`], then writes final assets, themes, manifests and records.

pub mod scene;
pub mod template;
pub mod profile;
pub mod post_export;
pub mod records;
pub mod set_json;
pub mod build;
