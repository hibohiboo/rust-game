use anyhow::{anyhow, Result};
use web_sys::{AudioBuffer, AudioBufferSourceNode, AudioContext, AudioDestinationNode, AudioNode};

pub fn create_audo_context() -> Result<AudioContext> {
  web_sys::AudioContext::new().map_err(|err| anyhow!("Failed to create audio context: {:#?}", err))
}

fn craete_buffer_source(context: &AudioContext) -> Result<AudioBufferSourceNode> {
  context
    .create_buffer_source()
    .map_err(|err| anyhow!("Failed to create buffer source: {:#?}", err))
}

fn connect_with_audio_node(source: &AudioBufferSourceNode, destination: &AudioDestinationNode) -> Result<AudioNode> {
  source
    .connect_with_audio_node(&destination)
    .map_err(|err| anyhow!("Failed to connect source with destination: {:#?}", err))
}
