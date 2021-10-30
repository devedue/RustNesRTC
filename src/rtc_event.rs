use crate::gui::Message;
use crate::rtc::DATA_CHANNEL_RX;
use iced_futures::futures;
use crate::nes::SPRITE_ARR_SIZE;
const MESSAGE_SIZE: usize = SPRITE_ARR_SIZE;

#[derive(Debug, Clone)]
pub enum RtcEvent {
    Message(Vec<u8>),
    Connected,
    CloseConnection,
    Waiting,
}

pub struct RtcEventRecipe {}

impl<H, I> iced_futures::subscription::Recipe<H, I> for RtcEventRecipe
where
    H: std::hash::Hasher,
{
    type Output = Message;

    fn hash(&self, state: &mut H) {
        use std::hash::Hash;
        std::any::TypeId::of::<Self>().hash(state);
        "Random Hash".hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: futures::stream::BoxStream<'static, I>,
    ) -> futures::stream::BoxStream<'static, Self::Output> {
        Box::pin(futures::stream::unfold(
            RtcEvent::Waiting,
            |state| async move {
                let mut buffer = vec![0u8; MESSAGE_SIZE];
                match state {
                    RtcEvent::CloseConnection => None,
                    RtcEvent::Waiting => {
                        let data_channel = DATA_CHANNEL_RX.lock().await;
                        match data_channel.clone() {
                            Some(_) => {
                                return Some((
                                    Message::RtcEvent(RtcEvent::Connected),
                                    RtcEvent::Connected,
                                ));
                            }
                            None => {
                                std::thread::sleep(std::time::Duration::from_secs(1));
                                return Some((
                                    Message::RtcEvent(RtcEvent::Waiting),
                                    RtcEvent::Waiting,
                                ));
                            }
                        };
                    }
                    RtcEvent::Message(_) | RtcEvent::Connected => {
                        let data_channel = DATA_CHANNEL_RX.lock().await;
                        let data_channel = match data_channel.clone() {
                            Some(dc) => dc,
                            None => {
                                std::thread::sleep(std::time::Duration::from_secs(1));
                                return Some((
                                    Message::RtcEvent(RtcEvent::Waiting),
                                    RtcEvent::Waiting,
                                ));
                            }
                        };
                        match data_channel.read(&mut buffer).await {
                            Ok(..) => {
                                return Some((
                                    Message::RtcEvent(RtcEvent::Message(buffer)),
                                    RtcEvent::Connected,
                                ));
                            }
                            Err(err) => {
                                println!("Datachannel closed; Exit the read_loop: {}", err);
                                return Some((
                                    Message::RtcEvent(RtcEvent::CloseConnection),
                                    RtcEvent::CloseConnection,
                                ));
                            }
                        };
                    }
                }
            },
        ))
    }
}
