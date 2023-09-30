use std::collections::{HashMap, HashSet};

use anyhow::Result;
use gloo_console::log;
use stylist::yew::styled_component;
use wasm_bindgen::JsValue;
//use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;
use yewdux::prelude::*;

use components::{
    CandidatesPanel, DismissButton, DismissablePanel, HistoryHalflife, ModeSelect, Selection,
    SimulationPanel,
};
//use nextspeaker::DEFAULT_HALFLIFE;
use state::AppMode;

mod components;
mod state;

const N_SIM: u64 = 1000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Mode {
    CandidatesView,
    HistoryView,
    MainView,
    SimulationView,
}

impl Default for Mode {
    fn default() -> Mode {
        Mode::MainView
    }
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

fn history_text_append(history_text: &str, new_selection: &str) -> String {
    let join = if history_text.is_empty() || history_text.ends_with('\n') {
        ""
    } else {
        "\n"
    };
    format!("{history_text}{join}{new_selection}")
}

#[derive(Default, Properties, PartialEq)]
pub struct AppProps {}

#[styled_component]
pub fn App(_props: &AppProps) -> Html {
    let (mode, mode_dispatch) = use_store::<AppMode>();
    let dismiss = mode_dispatch.reduce_mut_callback(|mode| mode.value = Mode::MainView);
    let candidates = mode_dispatch.reduce_mut_callback(|mode| mode.value = Mode::CandidatesView);
    let history = mode_dispatch.reduce_mut_callback(|mode| mode.value = Mode::HistoryView);
    let simulation = mode_dispatch.reduce_mut_callback(|mode| mode.value = Mode::SimulationView);
    let mode_select_buttons = html! {
        <div>
            <button onclick={candidates}>{"candidates"}</button>
            <button onclick={history}>{"history"}</button>
            <button onclick={simulation}>{"simulate"}</button>
        </div>
    };
    let inner = match mode.value {
        Mode::MainView => {
            html! {
                <div>
                    <ModeSelect buttons={mode_select_buttons} />
                </div>
            }
        }
        Mode::SimulationView => {
            let inner = html! {
                <SimulationPanel />
            };
            html! {
                <DismissablePanel dismiss={dismiss} children={inner} />
            }
        }
        Mode::CandidatesView => {
            let inner = html! {
                <CandidatesPanel />
            };
            html! {
                <DismissablePanel dismiss={dismiss} children={inner} />
            }
        }
        _ => {
            html! {
                <DismissablePanel dismiss={dismiss} children={html!{<p>{"hello not main"}</p>}} />
            }
        }
    };
    html! {
        <div>
            <h2>{"Rock 'n Roll!"}</h2>
            {inner}
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
