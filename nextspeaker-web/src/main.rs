use anyhow::Result;
use gloo_console::log;
use wasm_bindgen::JsValue;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

use localstore::LocalStore;

mod localstore;

const NEXTSPEAKER_KEY: &str = "It's next speaker by ed.cashin@acm.org!";

enum Msg {
    CandidatesUpdate(String),
    Choose,
    HistoryUpdate(String),
}

struct Model {
    candidates: Option<String>,
    history: Option<String>,
    local_store: LocalStore,
    selected: Option<String>,
}

fn from_lines(text: &str) -> Result<Vec<String>> {
    Ok(text
        .lines()
        .filter(|i| !i.is_empty())
        .map(|s| s.to_string())
        .collect())
}

#[derive(Properties, PartialEq)]
struct TextProps {
    oninput: Callback<InputEvent>,
    text: String,
}

#[function_component]
fn Text(props: &TextProps) -> Html {
    html! {
        <textarea value={props.text.clone()} oninput={props.oninput.clone()}></textarea>
    }
}

impl Model {
    fn save(&mut self) {
        let c = if let Some(c) = &self.candidates {
            c
        } else {
            ""
        };
        let h = if let Some(h) = &self.history { h } else { "" };
        let json = serde_json::to_string(&vec![c, h]).unwrap();
        self.local_store.save(&json).unwrap();
    }
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let local_store = LocalStore::new(NEXTSPEAKER_KEY, "").unwrap();
        let texts: Vec<String> = match serde_json::from_str(&local_store.value()) {
            Ok(t) => t,
            Err(e) => {
                log!(format!("cannot load local storage: {e}"));
                vec!["".to_owned(), "".to_owned()]
            }
        };
        let (candidates, history) = if texts.len() != 2 {
            log!("found bad local storage data");
            ("".to_owned(), "".to_owned())
        } else {
            (texts[0].clone(), texts[1].clone())
        };
        Self {
            candidates: Some(candidates),
            history: Some(history),
            local_store,
            selected: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::CandidatesUpdate(v) => {
                self.candidates = Some(v);
                self.save();
            }
            Msg::Choose => {
                let history_text = if let Some(h) = &self.history { h } else { "" };
                if let Some(candidates) = &self.candidates {
                    let candidates = from_lines(candidates).unwrap();
                    let history = from_lines(history_text).unwrap();
                    let selected = nextspeaker::choose(&candidates, &history, 10.0).unwrap();
                    self.history = Some(history_text_append(history_text, &selected));
                    self.selected = Some(selected);
                    log!(JsValue::from(self.selected.as_ref()));
                    self.save();
                }
            }
            Msg::HistoryUpdate(v) => {
                self.history = Some(v);
                self.save();
            }
        };
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let candidates_oninput = ctx.link().callback(|e: InputEvent| {
            let input: HtmlTextAreaElement = e.target_unchecked_into::<HtmlTextAreaElement>();
            Msg::CandidatesUpdate(input.value())
        });
        let history_oninput = ctx.link().callback(|e: InputEvent| {
            let input: HtmlTextAreaElement = e.target_unchecked_into::<HtmlTextAreaElement>();
            Msg::HistoryUpdate(input.value())
        });
        let onchoose = ctx.link().callback(|_| Msg::Choose);
        let candidates_text = if let Some(c) = &self.candidates {
            c
        } else {
            ""
        }
        .to_owned();
        let history_text = if let Some(h) = &self.history { h } else { "" }.to_owned();
        let selection = if let Some(s) = &self.selected {
            html! {
                <p>{ s.clone() }</p>
            }
        } else {
            html! {}
        };
        html! {
            <div class="content-area">
                <Text text={candidates_text.clone()} oninput={candidates_oninput}></Text>
                <Text text={history_text.clone()} oninput={history_oninput}></Text>
                <div class="action-area">
                    <button onclick={onchoose}>{"CHOOSE"}</button>
                </div>
                <div class="selection-display">
                    { selection }
                </div>
            </div>
        }
    }
}

fn history_text_append(history_text: &str, new_selection: &str) -> String {
    let join = if history_text.is_empty() || history_text.ends_with('\n') {
        ""
    } else {
        "\n"
    };
    format!("{history_text}{join}{new_selection}")
}

fn main() {
    yew::Renderer::<Model>::new().render();
}
