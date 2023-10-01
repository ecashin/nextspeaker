use stylist::yew::styled_component;
use yew::prelude::*;
use yew_router::prelude::*;

use components::{
    CandidatesPanel, ChooseButton, DismissablePanel, HistoryPanel, ModeSelect, Selection,
    SimulationPanel,
};

mod components;
mod simulate;
mod state;

const N_SIM: u64 = 1000;

#[derive(Clone, Debug, Eq, PartialEq, Routable)]
pub enum Mode {
    #[at("/candidates")]
    CandidatesView,
    #[at("/history")]
    HistoryView,
    #[at("/")]
    MainView,
    #[at("/simulation")]
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
    html! {
        <BrowserRouter>
            <h2>{"Rock 'n Roll!"}</h2>
            <Switch<Mode> render={switch} />
        </BrowserRouter>
    }
}

fn switch(mode: Mode) -> Html {
    match mode {
        Mode::MainView => {
            html! {
                <div>
                    <ModeSelect />
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
                <DismissablePanel children={inner} />
            }
        }
        Mode::CandidatesView => {
            let inner = html! {
                <CandidatesPanel />
            };
            html! {
                <DismissablePanel children={inner} />
            }
        }
        Mode::HistoryView => {
            let inner = html! {
                <HistoryPanel />
            };
            html! {
                <DismissablePanel children={inner} />
            }
        }
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
