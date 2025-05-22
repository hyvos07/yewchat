use crate::services::event_bus::EventBus;
use crate::{services::websocket::WebsocketService, User};

use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
struct UserProfile {
    name: String,
    avatar: String,
    bio: String,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    wss: WebsocketService,
    messages: Vec<MessageData>,
    _producer: Box<dyn Bridge<EventBus>>,
}

impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();
        let bio = user.bio.borrow().clone();

        log::info!("Registering user: {:?} with bio: {:?}", username, bio);

        let user: UserProfile = UserProfile {
            name: username.clone(),
            avatar:
            if username == "Miku" || username == "miku" {
                "https://raw.githubusercontent.com/hyvos07/yewchat/refs/heads/main/YewChat/static/miku.png".to_string()
            } else if username == "Teto" || username == "teto" {
                "https://raw.githubusercontent.com/hyvos07/yewchat/refs/heads/main/YewChat/static/teto.jpg".to_string()
            } else if username == "Neru" || username == "neru" {
                "https://raw.githubusercontent.com/hyvos07/yewchat/refs/heads/main/YewChat/static/neru.jpg".to_string()
            } else {
                "https://api.dicebear.com/9.x/lorelei/svg".to_string()
            },
            bio: bio
        };

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(serde_json::to_string(&user.clone()).unwrap()),
            data_array: None
        };

        log::info!("Sending registration message: {:?}", message);

        match serde_json::to_string(&message) {
            Ok(json) => {
                if let Err(e) = wss.tx.clone().try_send(json) {
                    log::error!("Failed to send registration message: {:?}", e);
                } else {
                    log::info!("Registration message sent successfully");
                }
            }
            Err(e) => log::error!("Failed to serialize message: {:?}", e),
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| {
                                let profile: UserProfile = serde_json::from_str(u).unwrap();
                                profile
                            })
                            .collect();
                        return true;
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        return true;
                    }
                    _ => {
                        return false;
                    }
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    log::debug!("got input: {:?}", input.value());
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(input.value()),
                        data_array: None,
                    };
                    if let Err(e) = self
                        .wss
                        .tx
                        .clone()
                        .try_send(serde_json::to_string(&message).unwrap())
                    {
                        log::debug!("error sending to channel: {:?}", e);
                    }
                    input.set_value("");
                };
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);
        let (user_context, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("User context to be set");
        let current_username = user_context.username.borrow().clone();

        html! {
            <div class="flex w-screen">
                <div class="flex-none w-56 h-screen bg-zinc-700">
                    <div class="text-xl p-3 text-white">{"Users"}</div>
                    {
                        self.users.clone().iter().map(|u| {
                            html!{
                                <div class="flex m-3 bg-white rounded-lg p-2">
                                    <div>
                                        <img class="w-12 h-12 rounded-full" src={u.avatar.clone()} alt="avatar"/>
                                    </div>
                                    <div class="flex-grow p-3">
                                        <div class="flex text-xs justify-between">
                                            <div>{u.name.clone()}</div>
                                        </div>
                                        <div class="text-xs text-gray-400">
                                            {u.bio.clone()}
                                        </div>
                                    </div>
                                </div>
                            }
                        }).collect::<Html>()
                    }
                </div>
                <div class="grow h-screen flex flex-col">
                    <div class="w-full h-14 border-b-2 border-gray-300"><div class="text-xl p-3">{"💬 Chat!"}</div></div>
                    <div class="w-full grow overflow-auto border-b-2 border-gray-300">
                        {
                            self.messages.iter().map(|m| {
                                let user = self.users.iter().find(|u| u.name == m.from).unwrap();
                                let is_current_user = m.from == current_username;
                                let message_class = if is_current_user {
                                    "flex items-end w-3/6 bg-blue-100 m-8 ml-auto rounded-tl-lg rounded-tr-lg rounded-bl-lg"
                                } else {
                                    "flex items-end w-3/6 bg-gray-100 m-8 rounded-tl-lg rounded-tr-lg rounded-br-lg"
                                };
                                html!{
                                    <div class={message_class}>
                                        <img class="w-8 h-8 rounded-full m-3" src={user.avatar.clone()} alt="avatar"/>
                                        <div class="p-3">
                                            <div class="text-sm">
                                                {m.from.clone()}
                                            </div>
                                            <div class="text-xs text-gray-500">
                                                if m.message.ends_with(".gif") {
                                                    <img class="mt-3" src={m.message.clone()}/>
                                                } else {
                                                    {m.message.clone()}
                                                }
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }

                    </div>
                    <div class="w-full h-14 flex px-3 items-center">
                        <input 
                            ref={self.chat_input.clone()} 
                            type="text" 
                            placeholder="Message" 
                            class="block w-full py-2 pl-4 mx-3 bg-gray-100 rounded-full outline-none focus:text-gray-700" 
                            name="message" 
                            required={true}
                            onkeypress={ctx.link().batch_callback(|e: KeyboardEvent| {
                                if e.key() == "Enter" {
                                    Some(Msg::SubmitMessage)
                                } else {
                                    None
                                }
                            })}
                        />
                        <button onclick={submit} class="p-3 shadow-sm bg-blue-600 w-10 h-10 rounded-full flex justify-center items-center color-white">
                            <svg fill="#000000" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" class="fill-white">
                                <path d="M0 0h24v24H0z" fill="none"></path><path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"></path>
                            </svg>
                        </button>
                    </div>
                </div>
            </div>
        }
    }
}
