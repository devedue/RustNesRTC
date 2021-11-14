use std::sync::Arc;
use tokio::sync::Mutex;
use webrtc_data::data_channel::DataChannel;

pub mod client;
pub mod server;


lazy_static! {
    pub static ref DATA_CHANNEL_RX: Arc<Mutex<Option<Arc<DataChannel>>>> = Arc::new(Mutex::new(None));
    pub static ref DATA_CHANNEL_TX: Arc<Mutex<Option<Arc<DataChannel>>>> = Arc::new(Mutex::new(None));
    pub static ref AUDIO_CHANNEL_RX: Arc<Mutex<Option<Arc<DataChannel>>>> = Arc::new(Mutex::new(None));
    pub static ref AUDIO_CHANNEL_TX: Arc<Mutex<Option<Arc<DataChannel>>>> = Arc::new(Mutex::new(None));
}