use std::collections::HashMap;

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
const N_SIM: i32 = 1000;
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
    RunSimulation,
}

enum Mode {
    CandidatesView,
    HistoryView,
    MainView,
    SimulationView,
}

struct Model {
    candidates: Option<String>,
    history: Option<String>,
    history_halflife: f64,
    local_store: LocalStore,
    mode: Mode,
    selected: Option<String>,
    simulation_results: Option<Vec<(String, u64)>>,
}

fn from_lines(text: &str) -> Result<Vec<String>> {
    Ok(text
        .lines()
        .filter(|i| !i.is_empty())
        .map(|s| s.to_string())
        .collect())
}

#[derive(Properties, PartialEq)]
struct SimulationPanelProps {
    dismiss: Callback<MouseEvent>,
    simulate: Callback<MouseEvent>,
    results: Option<Vec<(String, u64)>>,
}

#[styled_component]
fn SimulationPanel(props: &SimulationPanelProps) -> Html {
    html! {
        <div class={css!("display: flex; background-color: lightgray; flex-direction: column;")}>
            <div class={css!("display: flex; flex-flow: row-reverse;")}>
                <DismissButton onclick={props.dismiss.clone()} />
                <div class={css!("flex: 1;")} />
            </div>
            <div><h2>{"Simulation of Next Choice"}</h2></div>
            <div>
                <button onclick={props.simulate.clone()}>
                    {format!("run simulation {N_SIM} times")}
                </button>
                <SimulationResults results={props.results.clone()} />
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct SimulationResultsProps {
    results: Option<Vec<(String, u64)>>,
}

#[styled_component]
fn SimulationResults(props: &SimulationResultsProps) -> Html {
    if let Some(results) = &props.results {
        html! {
            <table>
                <tr><th>{"candidate"}</th><th width={"80%"}>{"selection count"}</th></tr>
                {
                    results.into_iter().map(|(candidate, count)| {
                        let pct = 100.0 * (*count as f64 / N_SIM as f64);
                        let pct = format!("{pct}%");
                        let bar_style = css!(
                            min-width: ${pct};
                            box-sizing: border-box;
                            flex: 0 0 auto;
                            color: darkgray;
                            background-color: darkgray;
                        );
                        html! {
                            <tr key={candidate.clone()}>
                                <td>{candidate}</td>
                                <td>
                                <div
                                    class={css!("background-color: whitesmoke; display: flex; flex-flow: row; width: 80%;")}
                                >
                                    <div class={bar_style}>{"X"}</div>
                                    <div class={css!("background-color: whitesmoke; flex: 1;")}></div>
                                </div>
                                </td>
                            </tr>
                        }
                    }).collect::<Html>()
                }
            </table>
        }
    } else {
        html! {
            <div>
                {"no results yet"}
            </div>
        }
    }
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
struct DismissButtonProps {
    onclick: Callback<MouseEvent>,
}

#[styled_component]
fn DismissButton(props: &DismissButtonProps) -> Html {
    html! {
        <button
            class={css!("color: red; justify-self: right; align-self: start; height: 1.6rem;")}
            onclick={props.onclick.clone()}
        >{"X"}</button>
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
            <DismissButton onclick={props.dismiss.clone()}></DismissButton>
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

fn sorted_counts(counts: HashMap<String, u64>) -> Vec<(String, u64)> {
    let mut count_vec = counts.into_iter().collect::<Vec<_>>();
    count_vec.sort_by(|(n1, _), (n2, _)| n1.cmp(n2));
    count_vec
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
            simulation_results: None,
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
            Msg::RunSimulation => {
                let history_text = if let Some(h) = &self.history { h } else { "" };
                if let Some(candidates) = &self.candidates {
                    let candidates = from_lines(candidates).unwrap();
                    let history = from_lines(history_text).unwrap();
                    let mut counts: HashMap<String, u64> =
                        candidates.iter().map(|c| (c.clone(), 0)).collect();
                    for _ in 0..N_SIM {
                        let selected =
                            nextspeaker::choose(&candidates, &history, self.history_halflife)
                                .unwrap();
                        *counts.entry(selected).or_insert(0) += 1;
                    }
                    self.simulation_results = Some(sorted_counts(counts));
                }
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
        let run_simulation = ctx.link().callback(|_e: MouseEvent| Msg::RunSimulation);
        let simulate = ctx
            .link()
            .callback(|_e: MouseEvent| Msg::ChangeView(Mode::SimulationView));
        let mode_select_buttons = html! {
            <div>
                <button onclick={candidates_view.clone()}>{"candidates"}</button>
                <button onclick={history_view.clone()}>{"history"}</button>
                <button onclick={simulate.clone()}>{"simulate"}</button>
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
            Mode::SimulationView => {
                html! {
                    <SimulationPanel dismiss={dismiss} simulate={run_simulation} results={self.simulation_results.clone()} />
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
