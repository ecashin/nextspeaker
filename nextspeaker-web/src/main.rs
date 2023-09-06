use anyhow::Result;
use gloo_console::log;
use serde::{Deserialize, Serialize};
use stylist::yew::styled_component;
use wasm_bindgen::JsValue;
use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;

use localstore::LocalStore;
use nextspeaker::DEFAULT_HALFLIFE;

mod localstore;

const LOCAL_STATE_SCHEMA_VERSION: &str = "v0.1";
const NEXTSPEAKER_KEY: &str = "It's next speaker by ed.cashin@acm.org!";

#[derive(Serialize, Deserialize)]
struct StoredState {
    history_halflife: f64,
    candidates: String,
    history: String,
}

fn default_initial_state() -> StoredState {
    StoredState {
        candidates: "".to_owned(),
        history_halflife: DEFAULT_HALFLIFE,
        history: "".to_owned(),
    }
}

enum Msg {
    CandidatesUpdate(String),
    ChangeView(Mode),
    Choose,
    HistoryUpdate(String),
    HistoryHalflifeUpdate(String),
}

enum Mode {
    CandidatesView,
    HistoryView,
    MainView,
}

struct Model {
    candidates: Option<String>,
    history: Option<String>,
    history_halflife: f64,
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
struct HistoryHalflifeProps {
    oninput: Callback<InputEvent>,
    value: f64,
}

#[styled_component]
fn HistoryHalflife(props: &HistoryHalflifeProps) -> Html {
    let value = props.value.to_string();
    html! {
        <div>
            <label for={"hhl2die4"}>{"History halflife:"}</label>
            <input
                type={"number"}
                id={"hhl2die4"}
                value={value}
                step={"any"}
                oninput={props.oninput.clone()}
            />
        </div>
    }
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
    buttons: Html,
}

#[styled_component]
fn ModeSelect(props: &ModeSelectProps) -> Html {
    html! {
        <div class={css!("width: 50%; padding: 3rem; margin: 1rem;")}>
            { props.buttons.clone() }
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
        let candidates = if let Some(c) = &self.candidates {
            c
        } else {
            ""
        };
        let history = if let Some(h) = &self.history { h } else { "" };
        let ss = serde_json::to_string(&StoredState {
            candidates: candidates.to_owned(),
            history: history.to_owned(),
            history_halflife: self.history_halflife,
        })
        .unwrap();
        let json = serde_json::to_string(&vec![LOCAL_STATE_SCHEMA_VERSION, &ss]).unwrap();
        self.local_store.save(&json).unwrap();
    }
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let local_store = LocalStore::new(NEXTSPEAKER_KEY, "").unwrap();
        let state: StoredState = match serde_json::from_str::<Vec<String>>(&local_store.value()) {
            Ok(t) => {
                if t.len() > 1 && t[0] == LOCAL_STATE_SCHEMA_VERSION {
                    match serde_json::from_str(&t[1]) {
                        Ok(stored_state) => stored_state,
                        Err(e) => {
                            log!(format!("cannot load local storage: {e}"));
                            default_initial_state()
                        }
                    }
                } else {
                    log!("mismatched local storage---aborting");
                    None::<f64>.unwrap(); // Don't want to overwrite existing state.
                    default_initial_state() // (unreached)
                }
            }
            Err(e) => {
                log!(format!("cannot load local storage: {e}"));
                default_initial_state()
            }
        };
        Self {
            candidates: Some(state.candidates),
            history: Some(state.history),
            history_halflife: state.history_halflife,
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
            Msg::ChangeView(mode) => {
                self.mode = mode;
            }
            Msg::Choose => {
                let history_text = if let Some(h) = &self.history { h } else { "" };
                if let Some(candidates) = &self.candidates {
                    let candidates = from_lines(candidates).unwrap();
                    let history = from_lines(history_text).unwrap();
                    let selected =
                        nextspeaker::choose(&candidates, &history, self.history_halflife).unwrap();
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
            Msg::HistoryHalflifeUpdate(v) => match &v.parse::<f64>() {
                Ok(hh) => {
                    self.history_halflife = *hh;
                    self.save();
                }
                Err(e) => {
                    log!(JsValue::from(&format!("cannot parse {v} as f64: {e}")));
                }
            },
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
        let history_halflife_oninput = ctx.link().callback(|e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into::<HtmlInputElement>();
            Msg::HistoryHalflifeUpdate(input.value())
        });
        let onchoose = ctx.link().callback(|_| Msg::Choose);
        let candidates_text = if let Some(c) = &self.candidates {
            c
        } else {
            ""
        }
        .to_owned();
        let history_text = if let Some(h) = &self.history { h } else { "" }.to_owned();
        let candidates_view = ctx
            .link()
            .callback(|_e: MouseEvent| Msg::ChangeView(Mode::CandidatesView));
        let history_view = ctx
            .link()
            .callback(|_e: MouseEvent| Msg::ChangeView(Mode::HistoryView));
        let dismiss = ctx
            .link()
            .callback(|_e: MouseEvent| Msg::ChangeView(Mode::MainView));
        let mode_select_buttons = html! {
            <div>
                <button onclick={candidates_view.clone()}>{"candidates"}</button>
                <button onclick={history_view.clone()}>{"history"}</button>
            </div>
        };
        match self.mode {
            Mode::MainView => {
                html! {
                    <div class="content-area">
                        <ModeSelect buttons={mode_select_buttons}></ModeSelect>
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
                    <div>
                        <HistoryHalflife
                            value={self.history_halflife}
                            oninput={history_halflife_oninput}
                        ></HistoryHalflife>
                        <DismissableText
                            heading={"history".to_owned()}
                            text={history_text.clone()}
                            oninput={history_oninput}
                            dismiss={dismiss}
                        ></DismissableText>
                    </div>
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
