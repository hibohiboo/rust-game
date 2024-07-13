use anyhow::{anyhow, Result};
use js_sys::ArrayBuffer;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
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
fn create_track_source(context: &AudioContext, buffer: &AudioBuffer) -> Result<AudioBufferSourceNode> {
  let source = craete_buffer_source(&context)?;
  source.set_buffer(Some(&buffer));
  connect_with_audio_node(&source, &context.destination())?;
  Ok(source)
}
pub enum LOOPING {
  NO,
  YES,
}
pub fn play_sound(context: &AudioContext, buffer: &AudioBuffer, looping: LOOPING) -> Result<()> {
  let source = create_track_source(context, buffer)?;
  if matches!(looping, LOOPING::YES) {
    source.set_loop(true);
  }
  source
    .start()
    .map_err(|err| anyhow!("Failed to start source: {:#?}", err))
}

pub async fn decode_audio_data(ctx: &AudioContext, array_buffer: &ArrayBuffer) -> Result<AudioBuffer> {
  JsFuture::from(
    ctx
      .decode_audio_data(array_buffer)
      .map_err(|err| anyhow!("Could not decode audio from array buffer {:#?}", err))?,
  )
  .await
  .map_err(|err| anyhow!("Could not convert promise to future {:#?}", err))?
  .dyn_into()
  .map_err(|err| anyhow!("Could not cast into AudioBuffer {:#?}", err))
}
