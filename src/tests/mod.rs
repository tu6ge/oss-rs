#[cfg(feature = "core")]
mod client;

#[cfg(feature = "decode")]
mod traits;

#[cfg(feature = "core")]
mod errors;

#[cfg(feature = "core")]
pub(crate) mod object;
