pub mod resource;

#[cfg(not(feature = "stateless"))]
pub mod inmemory;

#[cfg(feature = "stateless")]
pub mod stateless;

#[cfg(not(feature = "stateless"))]
pub use crate::state::inmemory::YummyState;

#[cfg(feature = "stateless")]
pub use crate::state::stateless::YummyState;

#[cfg(test)]
mod test;
