use crate::nes::Nes;
use crate::nes::NES_PTR;
use crate::nes::SPRITE_ARR_SIZE;
use crate::rtc::client::start_client;
use crate::rtc::server::start_server;
use crate::rtc::DATA_CHANNEL_TX;
use crate::rtc_event::RtcEvent;
use crate::rtc_event::RtcEventRecipe;
use crate::screen::Screen;
use hyper::body::Bytes;
use iced::time;
use iced::{
    button, executor, text_input, Application, Button, Clipboard, Column, Command, Container,
    Element, HorizontalAlignment, Length, Row, Settings, Subscription, Text, TextInput,
};
use std::time::{Duration, Instant};

use iced_aw::{modal, Card, Modal};

#[derive(Clone, Debug)]
pub enum DialogMessage {
    CloseModal,
}

#[derive(Default)]
struct DialogState {
    ok_state: button::State,
}

#[derive(Default)]
pub struct State {
    sdp: String,
    rom: String,
    connection_status: Connection,
    ti_sdp: text_input::State,
    bt_copy: button::State,
    bt_generate: button::State,
    bt_connect: button::State,
    bt_start: button::State,
    bt_stop: button::State,
    bt_browse: button::State,
    modal_state: modal::State<DialogState>,
    message_count: u64,
    key_state: u8,
    screen: Screen,
    started: bool,
}

pub struct MainMenu {
    state: State,
}

#[derive(std::cmp::PartialEq)]
enum Connection {
    Client,
    Server,
    Unspecified,
}

impl Default for Connection {
    fn default() -> Self {
        Connection::Unspecified
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    GenerateSDP,
    CopySDP,
    IPChanged(String),
    BrowseRom,
    StartNes,
    StopNes,
    Connect,
    RtcEvent(RtcEvent),
    DialogEvent(DialogMessage),
    Tick(Instant),
    NativeEvent(iced_native::Event),
}

impl MainMenu {
    pub fn start_program() {
        MainMenu::run(Settings {
            window: iced::window::Settings {
                size: (600, 600),
                ..iced::window::Settings::default()
            },
            ..Settings::default()
        })
        .unwrap();
    }
}

impl Application for MainMenu {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (MainMenu, Command<Message>) {
        (
            MainMenu {
                state: State::default(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Messenger")
    }

    fn view(&mut self) -> Element<Message> {
        let state = &mut self.state;
        let sdp_block = Row::new()
            .push(
                TextInput::new(
                    &mut state.ti_sdp,
                    "IP:PORT",
                    &mut state.sdp,
                    Message::IPChanged,
                )
                .padding(5),
            )
            // .push(Text::new("My IP: " + local_ip_address::local_ip().unwrap()))
            .push(Button::new(&mut state.bt_copy, Text::new("Copy")).on_press(Message::CopySDP))
            .push(
                Button::new(&mut state.bt_generate, Text::new("Server"))
                    .on_press(Message::GenerateSDP),
            )
            .push(
                Button::new(&mut state.bt_connect, Text::new("Connect")).on_press(Message::Connect),
            );

        let input_block = Row::new()
            .push(
                Button::new(&mut state.bt_browse, Text::new("Browse")).on_press(Message::BrowseRom),
            )
            .push(
                Button::new(
                    &mut state.bt_start,
                    Text::new(if state.started { "Restart" } else { "Start" }),
                )
                .on_press(Message::StartNes),
            )
            .push(Button::new(&mut state.bt_stop, Text::new("Stop")).on_press(Message::StopNes))
            .push(Text::new(&state.rom));

        let canvas = state.screen.view();

        let content = Column::new().push(sdp_block).push(input_block).push(canvas);

        let main_content = Container::new(content)
            .width(Length::Shrink)
            .height(Length::Shrink)
            .center_x()
            .center_y();

        Modal::new(&mut state.modal_state, main_content, |state| {
            Card::new(
                Text::new("Invalid Value"),
                Text::new("Enter a valid address in ip:port format"),
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

    fn subscription(&self) -> Subscription<Message> {
        let state = &self.state;
        match state.connection_status {
            Connection::Client => Subscription::batch([
                Subscription::from_recipe(RtcEventRecipe {}),
                time::every(Duration::from_millis(1000)).map(Message::Tick),
                iced_native::subscription::events().map(Message::NativeEvent),
            ]),
            Connection::Server => Subscription::batch([
                Subscription::from_recipe(RtcEventRecipe {}),
                time::every(Duration::from_millis(10)).map(Message::Tick),
                iced_native::subscription::events().map(Message::NativeEvent),
            ]),
            _ => Subscription::batch([
                time::every(Duration::from_millis(10)).map(Message::Tick),
                iced_native::subscription::events().map(Message::NativeEvent),
            ]),
        }
    }

    fn update(&mut self, message: Self::Message, clipboard: &mut Clipboard) -> Command<Message> {
        let mut state = &mut self.state;
        match message {
            Message::BrowseRom => {
                match tinyfiledialogs::open_file_dialog(
                    "Open",
                    "password.txt",
                    Some((&["*.nes"], "NES Rom")),
                ) {
                    Some(file) => state.rom = file,
                    None => state.rom = "null".to_string(),
                }
            }
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
                    state.connection_status = Connection::Client;
                }
            }
            Message::CopySDP => {
                if state.sdp.len() > 0 {
                    clipboard.write(state.sdp.to_owned());
                }
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
                    state.connection_status = Connection::Server;
                }
            }
            Message::IPChanged(value) => {
                state.sdp = value;
            }
            Message::StartNes => {
                if state.rom.is_empty() && state.connection_status == Connection::Server {
                    return Command::none();
                } else {
                    let mut nes = NES_PTR.lock().unwrap();
                    (*nes) = Nes::new(&state.rom);
                    drop(nes);
                    if state.connection_status != Connection::Client {
                        state.screen.init_nes();
                    }
                    if !state.started {
                        state
                            .screen
                            .run_nes(state.connection_status == Connection::Client);
                        state.started = true;
                    }
                }
            }
            Message::StopNes => {
                state.screen.stop_nes();
                state.started = false;
            }
            Message::RtcEvent(event) => match event {
                RtcEvent::Message(message) => {
                    if state.connection_status == Connection::Client {
                        let mut nes = NES_PTR.lock().unwrap();
                        // println!("mess {}",message.len());
                        (*nes).pal_positions = message;
                        state.message_count += 1;
                        state.screen.request_redraw();
                    } else {
                        // println!("received {}", message[0]);
                        let mut nes = NES_PTR.lock().unwrap();
                        (*nes).set_controller_state(message[0], 1);
                    }
                }
                RtcEvent::Connected => {
                    if state.connection_status == Connection::Client {
                    } else {
                    }
                }
                _ => {}
            },
            Message::DialogEvent(event) => match event {
                _ => {
                    state.modal_state.show(false);
                }
            },
            Message::NativeEvent(event) => {
                match event {
                    iced_native::Event::Keyboard(event) => {
                        match event {
                            iced_native::keyboard::Event::KeyPressed {
                                key_code,
                                modifiers: _,
                            } => match key_code {
                                iced_native::keyboard::KeyCode::Up => {
                                    state.key_state |= 0x08;
                                }
                                iced_native::keyboard::KeyCode::Down => {
                                    state.key_state |= 0x04;
                                }
                                iced_native::keyboard::KeyCode::Left => {
                                    state.key_state |= 0x02;
                                }
                                iced_native::keyboard::KeyCode::Right => {
                                    state.key_state |= 0x01;
                                }
                                iced_native::keyboard::KeyCode::A => {
                                    state.key_state |= 0x20;
                                }
                                iced_native::keyboard::KeyCode::S => {
                                    state.key_state |= 0x10;
                                }
                                iced_native::keyboard::KeyCode::Z => {
                                    state.key_state |= 0x40;
                                }
                                iced_native::keyboard::KeyCode::X => {
                                    state.key_state |= 0x80;
                                }
                                _ => {}
                            },
                            iced_native::keyboard::Event::KeyReleased {
                                key_code,
                                modifiers: _,
                            } => match key_code {
                                iced_native::keyboard::KeyCode::Up => {
                                    state.key_state &= !0x08;
                                }
                                iced_native::keyboard::KeyCode::Down => {
                                    state.key_state &= !0x04;
                                }
                                iced_native::keyboard::KeyCode::Left => {
                                    state.key_state &= !0x02;
                                }
                                iced_native::keyboard::KeyCode::Right => {
                                    state.key_state &= !0x01;
                                }
                                iced_native::keyboard::KeyCode::A => {
                                    state.key_state &= !0x20;
                                }
                                iced_native::keyboard::KeyCode::S => {
                                    state.key_state &= !0x10;
                                }
                                iced_native::keyboard::KeyCode::Z => {
                                    state.key_state &= !0x40;
                                }
                                iced_native::keyboard::KeyCode::X => {
                                    state.key_state &= !0x80;
                                }
                                _ => {}
                            },
                            _ => {}
                        }

                        match state.connection_status {
                            Connection::Client => {
                                let data = [state.key_state];
                                tokio::spawn(async move {
                                    let data_channel = DATA_CHANNEL_TX.lock().await;
                                    // println!("should send");
                                    let data_channel = match data_channel.clone() {
                                        Some(dc) => dc,
                                        None => {
                                            return Some((
                                                Message::RtcEvent(RtcEvent::Waiting),
                                                "".to_owned(),
                                            ));
                                        }
                                    };
                                    match data_channel.write(&Bytes::copy_from_slice(&data)).await {
                                        Ok(_) => {
                                            // println!("Sent {}", data[0]);
                                        }
                                        Err(err) => {
                                            println!("Not Sent, {}", err);
                                        }
                                    };
                                    None
                                });
                            }
                            _ => {
                                let mut nes = NES_PTR.lock().unwrap();
                                (*nes).set_controller_state(state.key_state, 0);
                            }
                        }
                        // println!("State: {}", state.key_state);
                    }
                    _ => {}
                }
            }
            Message::Tick(_) => {
                match state.connection_status {
                    Connection::Client => {
                        println!("Frames: {}", state.message_count);
                        state.message_count = 0;
                        // state.screen.request_redraw();
                    }
                    Connection::Server => {
                        let mut nes = NES_PTR.lock().unwrap();
                        let data = nes.get_pal_positions().to_owned();
                        drop(nes);
                        state.screen.request_redraw();
                        if data.len() >= SPRITE_ARR_SIZE {
                            tokio::spawn(async move {
                                // println!("REQUEST DATA LOCK");
                                let data_channel = DATA_CHANNEL_TX.lock().await;
                                // println!("DATA LOCK");
                                // println!("should send");
                                let data_channel = match data_channel.clone() {
                                    Some(dc) => dc,
                                    None => {
                                        // println!("Nos Data");
                                        return Some((
                                            Message::RtcEvent(RtcEvent::Waiting),
                                            "".to_owned(),
                                        ));
                                    }
                                };
                                match data_channel
                                    .write(&Bytes::copy_from_slice(
                                        data[0..SPRITE_ARR_SIZE].try_into().unwrap(),
                                    ))
                                    .await
                                {
                                    Ok(_) => {
                                        // println!("D");
                                    }
                                    Err(err) => {
                                        println!("Not Sent, {}", err);
                                    }
                                };
                                None
                            });
                        } else {
                            println!("Invalid Length");
                        }
                    }
                    Connection::Unspecified => {
                        let mut nes = NES_PTR.lock().unwrap();
                        let _data = nes.get_pal_positions().to_owned();
                        drop(nes);
                        state.screen.request_redraw();
                    }
                }
            }
        }

        Command::none()
    }
}
