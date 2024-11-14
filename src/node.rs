pub mod connection;
pub mod pool;

#[derive(serde::Serialize)]
pub struct SyncProgress {
    percentage: f64,
    era: u16,
    epoch: u32,
    slot: u64,
    block: String,
}
