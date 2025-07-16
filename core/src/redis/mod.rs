// mod client;  // Disabled for parsing exercise
// mod processor;  // Disabled for parsing exercise
mod local_emulator;

// Use local emulator for the parsing exercise
pub use local_emulator::{RelevantAccountCache, SmartAccountRedisClient};
