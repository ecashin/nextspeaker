use anyhow::Result;
use gloo_console::log;
use stylist::yew::styled_component;
use wasm_bindgen::JsValue;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

use localstore::LocalStore;

mod localstore;

const NEXTSPEAKER_KEY: &str = "It's next speaker by ed.cashin@acm.org!";

enum Msg {
    CandidatesUpdate(String),
    CandidatesView,
    Choose,
    HistoryUpdate(String),
    HistoryView,
    MainView,
}

enum Mode {
    CandidatesView,
    HistoryView,
    MainView,
}

struct Model {
    candidates: Option<String>,
    history: Option<String>,
    local_store: LocalStore,
    mode: Mode,
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
struct DismissableTextProps {
    heading: String,
    oninput: Callback<InputEvent>,
    dismiss: Callback<MouseEvent>,
    text: String,
}

#[styled_component]
fn DismissableText(props: &DismissableTextProps) -> Html {
    html! {
        <div class={css!("background-color: lightgray; display: grid; width: 90%; padding: 1rem; grid-template-columns: 80% 20%;")}>
            <Text heading={props.heading.clone()} text={props.text.clone()} oninput={props.oninput.clone()}></Text>
            <button
                class={css!("color: red; justify-self: right; align-self: start; height: 1.6rem;")}
                onclick={props.dismiss.clone()}
            >{"X"}</button>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct ModeSelectProps {
    candidates_view: Callback<MouseEvent>,
    history_view: Callback<MouseEvent>,
}

#[styled_component]
fn ModeSelect(props: &ModeSelectProps) -> Html {
    html! {
        <div class={css!("width: 50%; padding: 3rem; margin: 1rem;")}>
            <button onclick={props.candidates_view.clone()}>{"candidates"}</button>
            <button onclick={props.history_view.clone()}>{"history"}</button>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct TextProps {
    heading: String,
    oninput: Callback<InputEvent>,
    text: String,
}

#[styled_component]
fn Text(props: &TextProps) -> Html {
    html! {
        <div class={css!("width: 80%; margin: 3rem; height: 80%;")}>
            <h3>{props.heading.clone()}</h3>
            <textarea value={props.text.clone()} oninput={props.oninput.clone()}></textarea>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct SelectionProps {
    text: Option<String>,
}

#[function_component]
fn Selection(props: &SelectionProps) -> Html {
    if let Some(s) = &props.text {
        let text = format!("selection: {s}");
        html! {
            <div>
                <p>{text}</p>
            </div>
        }
    } else {
        html! {}
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
            mode: Mode::MainView,
            selected: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::CandidatesUpdate(v) => {
                self.candidates = Some(v);
                self.save();
            }
            Msg::CandidatesView => {
                self.mode = Mode::CandidatesView;
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
            Msg::MainView => {
                self.mode = Mode::MainView;
            }
            Msg::HistoryView => {
                self.mode = Mode::HistoryView;
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
        let candidates_view = ctx.link().callback(|_e: MouseEvent| Msg::CandidatesView);
        let history_view = ctx.link().callback(|_e: MouseEvent| Msg::HistoryView);
        let dismiss = ctx.link().callback(|_e: MouseEvent| Msg::MainView);
        match self.mode {
            Mode::MainView => {
                html! {
                    <div class="content-area">
                        <ModeSelect candidates_view={candidates_view} history_view={history_view}></ModeSelect>
                        <div class="action-area">
                            <button onclick={onchoose}>{"CHOOSE"}</button>
                        </div>
                        <div class="selection-display">
                            <Selection text={self.selected.clone()} />
                        </div>
                    </div>
                }
            }
            Mode::CandidatesView => {
                html! {
                    <DismissableText
                        heading={"candidates".to_owned()}
                        text={candidates_text.clone()}
                        oninput={candidates_oninput}
                        dismiss={dismiss.clone()}
                    ></DismissableText>
                }
            }
            Mode::HistoryView => {
                html! {
                    <DismissableText
                        heading={"history".to_owned()}
                        text={history_text.clone()}
                        oninput={history_oninput}
                        dismiss={dismiss}
                    ></DismissableText>
                }
            }
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
