mod error;
#[cfg(test)]
mod tests;
mod typed;

pub use error::MessageError;
pub use typed::generate_extension;
pub use typed::BundleMessageExt;
pub use typed::Message;
