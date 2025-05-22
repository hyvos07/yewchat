use web_sys::HtmlInputElement;
use yew::functional::*;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::Route;
use crate::User;

#[function_component(Login)]
pub fn login() -> Html {
    let username = use_state(|| String::new());
    let bio = use_state(|| String::new());
    let user = use_context::<User>().expect("No context found.");

    let onclick = {
        let username = username.clone();
        let bio = bio.clone();
        let user = user.clone();
        Callback::from(move |_| {
            user.username.replace(username.to_string());
            user.bio.replace(bio.to_string());
        })
    };

    html! {
        <div class="bg-gray-800 flex w-screen">
            <div class="container mx-auto flex flex-col justify-center items-center">
            <form class="m-4 flex flex-col gap-4">
                <input oninput={
                    let current_username = username.clone();

                    Callback::from(move |e: InputEvent| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        current_username.set(input.value());
                    })
                } class="rounded-lg p-4 border text-gray-800 border-gray-200 bg-white" placeholder="Username"/>
                
                <input oninput={
                    let current_bio = bio.clone();
                    Callback::from(move |e: InputEvent| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        current_bio.set(input.value());
                    })
                } class="rounded-lg p-4 border text-gray-800 border-gray-200 bg-white" placeholder="Bio"/>
                
                <Link<Route> to={Route::Chat}>
                    <button {onclick} disabled={username.len()<1} class="w-full rounded-lg bg-violet-600 text-white font-bold p-4 uppercase border-violet-600 border">
                        {"Go Chatting!"}
                    </button>
                </Link<Route>>
            </form>
            </div>
        </div>
    }
}
