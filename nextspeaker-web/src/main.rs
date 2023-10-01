use gloo_console::log;
use stylist::yew::styled_component;
use wasm_bindgen::JsValue;
use yew::prelude::*;
use yewdux::prelude::*;

use components::{
    CandidatesPanel, ChooseButton, DismissablePanel, HistoryPanel, ModeSelect, Selection,
    SimulationPanel,
};
use state::AppMode;

mod components;
mod simulate;
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

#[derive(Default, Properties, PartialEq)]
pub struct AppProps {}

#[styled_component]
pub fn App(_props: &AppProps) -> Html {
    let (mode, mode_dispatch) = use_store::<AppMode>();
    let dismiss = mode_dispatch.reduce_mut_callback(|mode| mode.value = Mode::MainView);
    let candidates = mode_dispatch.reduce_mut_callback(|mode| mode.value = Mode::CandidatesView);
    let history = mode_dispatch.reduce_mut_callback(|mode| mode.value = Mode::HistoryView);
    let simulation = mode_dispatch.reduce_mut_callback(|mode| {
        yew::platform::spawn_local(async {
            log!(JsValue::from("run sim thing"));
            let candidates = Dispatch::<state::Candidates>::new().get();
            let history = Dispatch::<state::History>::new().get();
            let history_halflife = Dispatch::<state::HistoryHalflife>::new().get().into_f64();
            let results = simulate::run(&candidates.value, &history.value, history_halflife);
            Dispatch::<state::SimulationResults>::new()
                .set(state::SimulationResults { value: results });
        });
        mode.value = Mode::SimulationView;
    });
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
                    <ChooseButton />
                    <Selection />
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
        Mode::HistoryView => {
            let inner = html! {
                <HistoryPanel />
            };
            html! {
                <DismissablePanel dismiss={dismiss} children={inner} />
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
