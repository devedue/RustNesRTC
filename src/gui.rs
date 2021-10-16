use iced::{button, executor, scrollable, text_input};
use iced::{
    Application, Button, Clipboard, Column, Command, Container, Element, Length, Row, Scrollable,
    Settings, Text, TextInput,
};

#[derive(Default)]
pub struct State {
    sent: Vec<String>,
    received: Vec<String>,
    input_value: String,
    sdp: String,
    ti_message: text_input::State,
    ti_sdp: text_input::State,
    bt_copy: button::State,
    bt_generate: button::State,
    bt_connect: button::State,
    bt_send: button::State,
    scroll_send: scrollable::State,
    scroll_receive: scrollable::State,
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
        String::from("Counter")
    }

    fn view(&mut self) -> Element<Message> {
        match self {
            // MainMenu::Loading => {
            //     let content = Column::new().push(Text::new("Loading..."));
            //     Container::new(content)
            //         .width(Length::Shrink)
            //         .height(Length::Shrink)
            //         .center_x()
            //         .center_y()
            //         .into()
            // }
            MainMenu::Loaded(state) => {
                let sdp_block = Row::new()
                    .push(
                        TextInput::new(
                            &mut state.ti_sdp,
                            "SDP (Generate or Enter)",
                            &mut state.sdp,
                            Message::SDPChanged,
                        )
                        .padding(15)
                        .size(30)
                    )
                    .push(
                        Button::new(&mut state.bt_copy, Text::new("Copy"))
                            .on_press(Message::CopySDP),
                    )
                    .push(
                        Button::new(&mut state.bt_generate, Text::new("Generate"))
                            .on_press(Message::GenerateSDP),
                    )
                    .push(
                        Button::new(&mut state.bt_connect, Text::new("Connect"))
                            .on_press(Message::Connect),
                    );

                let messages_block =
                    Row::new()
                        .push(
                            Scrollable::new(&mut state.scroll_send).padding(40).push(
                                state.sent.iter_mut().enumerate().fold(
                                    Column::new().spacing(5),
                                    |column, (_i, message)| {
                                        column.push(Text::new(message.to_owned()))
                                    },
                                ),
                            ),
                        )
                        .push(
                            Scrollable::new(&mut state.scroll_receive).padding(40).push(
                                state.received.iter_mut().enumerate().fold(
                                    Column::new().spacing(5),
                                    |column, (_i, message)| {
                                        column.push(Text::new(message.to_owned()))
                                    },
                                ),
                            ),
                        );

                let input = TextInput::new(
                    &mut state.ti_message,
                    "Send Message",
                    &mut state.input_value,
                    Message::InputChanged,
                )
                .padding(15)
                .size(30)
                .on_submit(Message::SendMessage);

                let input_block = Row::new().push(input).push(
                    Button::new(&mut state.bt_send, Text::new("Send"))
                        .on_press(Message::SendMessage),
                );

                let content = Column::new()
                    .push(sdp_block)
                    .push(messages_block)
                    .push(input_block);

                Container::new(content)
                    .width(Length::Shrink)
                    .height(Length::Shrink)
                    .center_x()
                    .center_y()
                    .into()
            }
        }
    }

    fn update(&mut self, message: Self::Message, clipboard: &mut Clipboard) -> Command<Message> {
        match self {
            MainMenu::Loaded(state) => match message {
                Message::Connect => {}
                Message::CopySDP => {
                    clipboard.write(state.sdp.to_owned());
                }
                Message::GenerateSDP => {
                    state.sdp = "Generated".to_owned();
                }
                Message::InputChanged(value) => {
                    state.input_value = value;
                }
                Message::SDPChanged(value) => {
                    state.sdp = value;
                }
                Message::SendMessage => {
                    state.sent.push(state.input_value.to_owned());
                }
            },
        }

        Command::none()
    }
}
