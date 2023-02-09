extern crate core;

mod claim;
pub mod contract;
mod cw20_distributions;
mod distributing;
mod native_distributions;
mod rewards;
mod state;
mod update_weights;

#[cfg(test)]
mod tests;