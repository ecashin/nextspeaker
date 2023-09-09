use std::collections::{HashMap, HashSet};

use anyhow::Result;
use gloo_console::log;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;

use components::{DismissableText, HistoryHalflife, ModeSelect, Selection, SimulationPanel};
use localstore::LocalStore;
use nextspeaker::DEFAULT_HALFLIFE;

mod components;
mod localstore;

const LOCAL_STATE_SCHEMA_VERSION: &str = "v0.1";
const N_SIM: u64 = 1000;
const NEXTSPEAKER_KEY: &str = "It is next speaker by ed.cashin@acm.org!";

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

fn ignore_non_candidates(candidates: &Vec<String>, history: Vec<String>) -> Vec<String> {
    log!(JsValue::from(&format!("{:?}", candidates)));
    let candidates: HashSet<_> = candidates.iter().collect();
    log!(JsValue::from(&format!("{:?}", &history)));
    let history = history
        .into_iter()
        .filter(|h| candidates.contains(h))
        .collect();
    log!(JsValue::from(&format!("{:?}", &history)));
    history
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
                    let history = ignore_non_candidates(&candidates, history);
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
                    <div>
                        <ModeSelect buttons={mode_select_buttons}></ModeSelect>
                        <div>
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
