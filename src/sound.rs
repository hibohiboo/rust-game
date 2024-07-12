use anyhow::{anyhow, Result};
use web_sys::AudioContext;

pub fn create_audo_context() -> Result<AudioContext> {
  web_sys::AudioContext::new().map_err(|err| anyhow!("Failed to create audio context: {:#?}", err))
}
