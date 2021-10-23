use crate::nes::Nes;
use crate::rtc::client::start_client;
use crate::rtc::server::start_server;
use crate::rtc::DATA_CHANNEL_TX;
use crate::rtc_event::RtcEvent;
use crate::rtc_event::RtcEventRecipe;
use hyper::body::Bytes;
use iced::{
    button, executor, scrollable, text_input, Application, Button, Clipboard, Column, Command,
    Container, Element, HorizontalAlignment, Length, Row, Scrollable, Settings, Subscription, Text,
    TextInput,
};
use pge::PGE;

use iced_aw::{modal, Card, Modal};

#[derive(Clone, Debug)]
pub enum DialogMessage {
    CloseModal,
}

#[derive(Default)]
struct DialogState {
    ok_state: button::State,
}

pub struct MessageElement {
    sent: bool,
    message: String,
}

#[derive(Default)]
pub struct State {
    messages: Vec<MessageElement>,
    input_value: String,
    sdp: String,
    started: u8,
    ti_message: text_input::State,
    ti_sdp: text_input::State,
    bt_copy: button::State,
    bt_generate: button::State,
    bt_connect: button::State,
    bt_send: button::State,
    scroll_messages: scrollable::State,
    modal_state: modal::State<DialogState>,
}

pub enum MainMenu {
    Loaded(State),
}

#[derive(Debug, Clone)]
pub enum Message {
    GenerateSDP,
    CopySDP,
    InputChanged(String),
    SDPChanged(String),
    SendMessage,
    Connect,
    RtcEvent(RtcEvent),
    DialogEvent(DialogMessage),
}

impl MainMenu {
    pub fn start_program() {
        MainMenu::run(Settings::default()).unwrap();
    }
}

impl Application for MainMenu {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (MainMenu, Command<Message>) {
        (MainMenu::Loaded(State::default()), Command::none())
    }

    fn title(&self) -> String {
        String::from("Messenger")
    }

    fn view(&mut self) -> Element<Message> {
        match self {
            MainMenu::Loaded(state) => {
                let sdp_block = Row::new()
                    .push(
                        TextInput::new(
                            &mut state.ti_sdp,
                            "IP:PORT",
                            &mut state.sdp,
                            Message::SDPChanged,
                        )
                        .padding(5),
                    )
                    .push(
                        Button::new(&mut state.bt_copy, Text::new("Copy"))
                            .on_press(Message::CopySDP),
                    )
                    .push(
                        Button::new(&mut state.bt_generate, Text::new("Server"))
                            .on_press(Message::GenerateSDP),
                    )
                    .push(
                        Button::new(&mut state.bt_connect, Text::new("Connect"))
                            .on_press(Message::Connect),
                    );

                let messages_block = Row::new().height(Length::Fill).push(
                    Scrollable::new(&mut state.scroll_messages)
                        .padding(40)
                        .width(Length::Fill)
                        .push(state.messages.iter_mut().enumerate().fold(
                            Column::new().spacing(5),
                            |column, (_i, message)| {
                                column.push(
                                    Text::new(message.message.to_owned())
                                        .width(Length::Fill)
                                        .horizontal_alignment(if message.sent {
                                            HorizontalAlignment::Right
                                        } else {
                                            HorizontalAlignment::Left
                                        }),
                                )
                            },
                        )),
                );

                let input = TextInput::new(
                    &mut state.ti_message,
                    "Send Message",
                    &mut state.input_value,
                    Message::InputChanged,
                )
                .padding(5)
                .on_submit(Message::SendMessage);

                let input_block = Row::new().push(input).push(
                    Button::new(&mut state.bt_send, Text::new("Send"))
                        .on_press(Message::SendMessage),
                );

                let content = Column::new()
                    .push(sdp_block)
                    .push(messages_block)
                    .push(input_block);

                let main_content = Container::new(content)
                    .width(Length::Shrink)
                    .height(Length::Shrink)
                    .center_x()
                    .center_y();

                Modal::new(&mut state.modal_state, main_content, |state| {
                    Card::new(
                        Text::new("Invalid Value"),
                        Text::new("Enter a valid address in ip:port format"), //Text::new("Zombie ipsum reversus ab viral inferno, nam rick grimes malum cerebro. De carne lumbering animata corpora quaeritis. Summus brains sit​​, morbo vel maleficia? De apocalypsi gorger omero undead survivor dictum mauris. Hi mindless mortuis soulless creaturas, imo evil stalking monstra adventus resi dentevil vultus comedat cerebella viventium. Qui animated corpse, cricket bat max brucks terribilem incessu zomby. The voodoo sacerdos flesh eater, suscitat mortuos comedere carnem virus. Zonbi tattered for solum oculi eorum defunctis go lum cerebro. Nescio brains an Undead zombies. Sicut malus putrid voodoo horror. Nigh tofth eliv ingdead.")
                    )
                    .foot(
                        Row::new().spacing(10).padding(5).width(Length::Fill).push(
                            Button::new(
                                &mut state.ok_state,
                                Text::new("Ok").horizontal_alignment(HorizontalAlignment::Center),
                            )
                            .width(Length::Fill)
                            .on_press(Message::DialogEvent(DialogMessage::CloseModal)),
                        ),
                    )
                    .max_width(300)
                    //.width(Length::Shrink)
                    .on_close(Message::DialogEvent(DialogMessage::CloseModal))
                    .into()
                })
                .backdrop(Message::DialogEvent(DialogMessage::CloseModal))
                .on_esc(Message::DialogEvent(DialogMessage::CloseModal))
                .into()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        match self {
            MainMenu::Loaded(state) => match state.started {
                1 => Subscription::from_recipe(RtcEventRecipe {}),
                2 => Subscription::from_recipe(RtcEventRecipe {}),
                _ => Subscription::none(),
            },
        }
    }

    fn update(&mut self, message: Self::Message, clipboard: &mut Clipboard) -> Command<Message> {
        match self {
            MainMenu::Loaded(state) => match message {
                Message::Connect => {
                    let ip = state.sdp.clone();
                    if ip.is_empty() {
                        state.modal_state.show(true);
                    } else {
                        tokio::spawn(async {
                            if let Err(e) = start_client(ip).await {
                                eprintln!("server error: {}", e);
                            }
                        });
                        state.started = 1;
                    }
                }
                Message::CopySDP => {
                    clipboard.write(state.sdp.to_owned());
                }
                Message::GenerateSDP => {
                    let ip = state.sdp.clone();
                    if ip.is_empty() {
                        state.modal_state.show(true);
                    } else {
                        tokio::spawn(async {
                            if let Err(e) = start_server(ip).await {
                                eprintln!("server error: {}", e);
                            }
                        });
                        state.started = 2;
                    }
                }
                Message::InputChanged(value) => {
                    state.input_value = value;
                }
                Message::SDPChanged(value) => {
                    state.sdp = value;
                }
                Message::SendMessage => {
                    let message = state.input_value.clone();
                    state.messages.push(MessageElement {
                        sent: true,
                        message: message.clone(),
                    });
                    if state.started != 0 {
                        tokio::spawn(async {
                            let data_channel = DATA_CHANNEL_TX.lock().await;
                            let data_channel = match data_channel.clone() {
                                Some(dc) => dc,
                                None => {
                                    return Some((
                                        Message::RtcEvent(RtcEvent::Waiting),
                                        "".to_owned(),
                                    ));
                                }
                            };
                            match data_channel.write(&Bytes::from(message)).await {
                                Ok(_) => {}
                                Err(_) => {}
                            };
                            None
                        });
                    }
                }
                Message::RtcEvent(event) => match event {
                    RtcEvent::Message(message) => {
                        if !message.is_empty() {
                            let msg_str = String::from_utf8(message).unwrap();
                            state.messages.push(MessageElement {
                                sent: false,
                                message: msg_str.to_owned(),
                            });
                        }
                    }
                    RtcEvent::Connected => {
                        if state.started == 1 {
                            std::thread::spawn(move || {
                                let mut nes = Nes::new();
                                let mut pge = PGE::construct("NES Emulator", 512, 480, 2, 2);
                                pge.start(&mut nes);
                            });
                        } else {
                            std::thread::spawn(move || {
                                let mut nes = Nes::new();
                                let mut pge = PGE::construct("NES Emulator", 512, 480, 2, 2);
                                pge.start(&mut nes);
                            });
                        }
                    }
                    _ => {}
                },
                Message::DialogEvent(event) => match event {
                    _ => {
                        state.modal_state.show(false);
                    }
                },
            },
        }

        Command::none()
    }
}