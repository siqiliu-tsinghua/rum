//! G15-CM identity and counted requirements. Neither stage settles obligations.
mod applicability;
mod crosswalk;
mod discovery;
mod identity;
mod imports;
mod owners;
mod partition;
mod policy;
mod requirements;
mod resolve;
mod routes;

#[cfg(test)]
mod requirement_tests;

#[cfg(test)]
mod tests;
