//! PICS product info: which depots an app has, at which manifest ids.

mod app;
mod client;

pub use app::{AppInfo, Branch, Depot, DepotFilter, Os};
pub use client::{PicsError, product_info};
