use stylist::yew::styled_component;
use yew::prelude::*;
use yewdux::prelude::*;

use components::{
    CandidatesPanel, ChooseButton, HistoryPanel, ModeSelect, Selection, SimulationPanel,
};

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
    let (mode, _mode_dispatch) = use_store::<state::AppMode>();
    let inner = match mode.value {
        Mode::MainView => {
            html! {
                <div>
                    <ChooseButton />
                    <Selection />
                </div>
            }
        }
        Mode::SimulationView => {
            html! {
                <SimulationPanel />
            }
        }
        Mode::CandidatesView => {
            html! {
                <CandidatesPanel />
            }
        }
        Mode::HistoryView => {
            html! {
                <HistoryPanel />
            }
        }
    };
    html! {
        <div>
            <div class="content">
                <h2>{"Rock 'n Roll!"}</h2>
            </div>
            <ModeSelect />
            {inner}
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
