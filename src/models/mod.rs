pub mod material;
pub mod quote;
pub mod service_type;

// Re-exports for convenience - these may be used in future phases
#[allow(unused_imports)]
pub use material::Material;
pub use quote::QuoteSession;
#[allow(unused_imports)]
pub use quote::{Quote, UploadedModel};
#[allow(unused_imports)]
pub use service_type::ServiceType;
