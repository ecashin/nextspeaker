use std::collections::HashSet;

use gloo_console::log;
use stylist::yew::styled_component;
use wasm_bindgen::JsValue;
use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::simulate;
use crate::state;
use crate::Mode;
use crate::N_SIM;

fn ignore_non_candidates(candidates: &Vec<String>, history: &Vec<String>) -> Vec<String> {
    log!(JsValue::from(&format!("{:?}", candidates)));
    let candidates: HashSet<_> = candidates.iter().collect();
    log!(JsValue::from(&format!("{:?}", &history)));
    let history = history
        .into_iter()
        .filter(|h| candidates.contains(h))
        .cloned()
        .collect();
    log!(JsValue::from(&format!("{:?}", &history)));
    history
}

#[derive(Properties, PartialEq)]
pub struct ChooseButtonProps {}

#[styled_component]
pub fn ChooseButton(_props: &ChooseButtonProps) -> Html {
    let selected_dispatch = Dispatch::<state::Selected>::new();
    let candidates = Dispatch::<state::Candidates>::new().get();
    let (history, history_dispatch) = use_store::<state::History>();
    let history_halflife = {
        let hh = Dispatch::<state::HistoryHalflife>::new().get();
        hh.into_f64()
    };
    log!(JsValue::from(&format!(
        "history_halflife: {}",
        history_halflife
    )));
    let onclick = selected_dispatch.reduce_mut_callback(move |selected| {
        if !candidates.value.is_empty() {
            let history = ignore_non_candidates(&candidates.value, &history.value);
            let new_selection =
                nextspeaker::choose(&candidates.value, &history, history_halflife).unwrap();
            history_dispatch.reduce_mut(|h| h.value.push(new_selection.clone()));
            selected.value = new_selection;
        }
    });
    html! {
        <button class={"button is-rounded is-primary"} {onclick}>{"choose"}</button>
    }
}

#[derive(Properties, PartialEq)]

pub struct SimulationPanelProps {}

#[styled_component]
pub fn SimulationPanel(_props: &SimulationPanelProps) -> Html {
    html! {
        <div class="has-background-grey-light">
            <div class="content">
                <h2>{"Simulation of Next Choice"}</h2>
            </div>
            <SimulationResults />
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct SimulationBarProps {
    pub candidate: String,
    pub count: u64,
    pub total: u64,
}

#[styled_component]
pub fn SimulationBar(props: &SimulationBarProps) -> Html {
    let SimulationBarProps {
        candidate,
        count,
        total,
    } = props;
    let pct = 100.0 * (*count as f64 / *total as f64);
    let pct = format!("{pct}");
    html! {
        <tr key={candidate.clone()}>
            <td>{candidate}</td>
            <td>
                <progress class="progress" value={pct} max={"100"}>{*count}</progress>
            </td>
        </tr>
    }
}

#[derive(Properties, PartialEq)]
pub struct SimulationResultsProps {}

#[styled_component]
pub fn SimulationResults(_props: &SimulationResultsProps) -> Html {
    let (results, _) = use_store::<state::SimulationResults>();
    if let Some(results) = &results.value {
        html! {
            <table class={"is-striped"}>
                <tr><th>{"candidate"}</th><th>{"selection count"}</th></tr>
                {
                    results.iter().map(|(candidate, count)| {
                        html! {
                            <SimulationBar
                                candidate={candidate.clone()}
                                count={*count}
                                total={N_SIM}
                            />
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
pub struct HistoryHalflifeProps {}

#[styled_component]
pub fn HistoryHalflife(_props: &HistoryHalflifeProps) -> Html {
    let (hhl, dispatch) = use_store::<state::HistoryHalflife>();
    let oninput_numerator = dispatch.reduce_mut_callback_with(|hhl, e: InputEvent| {
        let input: HtmlInputElement = e.target_unchecked_into::<HtmlInputElement>();
        if let Ok(num) = input.value().parse::<i64>() {
            hhl.numerator = num;
        }
    });
    let oninput_denominator = dispatch.reduce_mut_callback_with(|hhl, e: InputEvent| {
        let input: HtmlInputElement = e.target_unchecked_into::<HtmlInputElement>();
        if let Ok(denom) = input.value().parse::<i64>() {
            hhl.denominator = if denom == 0 { 1 } else { denom };
        }
    });
    let hhl_text = format!("History halflife: {:.2}", hhl.into_f64());
    html! {
        <div class={"content"}>
            <h3>{hhl_text}</h3>
            <div class={"columns"}>
                <div class={"column"}>
                    <label for={"hhl2die4"}>{"History halflife numerator:"}</label>
                    <input
                        type={"number"}
                        id={"hhl2die4"}
                        value={hhl.numerator.to_string()}
                        min={"1"}
                        oninput={oninput_numerator}
                    />
                </div>
                <div class={"column"}>
                    <label for={"hhl2live4"}>{"History halflife denominator:"}</label>
                    <input
                        type={"number"}
                        id={"hhl2live4"}
                        value={hhl.denominator.to_string()}
                        min={"1"}
                        oninput={oninput_denominator}
                    />
                </div>
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct HistoryPanelProps {}

#[styled_component]
pub fn HistoryPanel(_props: &HistoryPanelProps) -> Html {
    let (history, history_dispatch) = use_store::<state::History>();
    let oninput = history_dispatch.reduce_mut_callback_with(|history, e: InputEvent| {
        let input: HtmlTextAreaElement = e.target_unchecked_into::<HtmlTextAreaElement>();
        history.value = from_lines(&input.value());
    });
    let content = history.value.join("\n");
    html! {
        <div>
            <Text heading={"history"} text={content} {oninput} />
            <HistoryHalflife />
        </div>
    }
}

fn from_lines(text: &str) -> Vec<String> {
    text.lines()
        .filter(|i| !i.is_empty())
        .map(|s| s.to_string())
        .collect()
}

#[derive(Properties, PartialEq)]
pub struct CandidatesPanelProps {}

#[styled_component]
pub fn CandidatesPanel(_props: &CandidatesPanelProps) -> Html {
    let (candidates, dispatch) = use_store::<state::Candidates>();
    let oninput = dispatch.reduce_mut_callback_with(|candidates, e: InputEvent| {
        let input: HtmlTextAreaElement = e.target_unchecked_into::<HtmlTextAreaElement>();
        candidates.value = from_lines(&input.value());
    });
    let content = candidates.value.join("\n");
    html! {
        <Text heading={"candidates"} text={content} {oninput} />
    }
}

#[derive(Properties, PartialEq)]
pub struct ModeSelectProps {}

#[styled_component]
pub fn ModeSelect(_props: &ModeSelectProps) -> Html {
    let (mode, mode_dispatch) = use_store::<state::AppMode>();
    let go_main = mode_dispatch.reduce_mut_callback(|mode| mode.value = Mode::MainView);
    let go_candidates = mode_dispatch.reduce_mut_callback(|mode| mode.value = Mode::CandidatesView);
    let go_history = mode_dispatch.reduce_mut_callback(|mode| mode.value = Mode::HistoryView);
    let go_simulation = mode_dispatch.reduce_mut_callback(|mode| {
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
    html! {
        <div class={"tabs"}>
            <ul>
                <li class={if mode.value == Mode::MainView { "is-active" } else { "" }}><a onclick={go_main}>{"Main"}</a></li>
                <li class={if mode.value == Mode::CandidatesView { "is-active" } else { "" }}><a onclick={go_candidates}>{"Candidates"}</a></li>
                <li class={if mode.value == Mode::HistoryView { "is-active" } else { "" }}><a onclick={go_history}>{"History"}</a></li>
                <li class={if mode.value == Mode::SimulationView { "is-active" } else { "" }}><a onclick={go_simulation}>{"Simulation"}</a></li>
            </ul>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct TextProps {
    pub heading: String,
    pub oninput: Callback<InputEvent>,
    pub text: String,
}

#[styled_component]
pub fn Text(props: &TextProps) -> Html {
    html! {
        <div>
            <div class="content">
                <h3>{props.heading.clone()}</h3>
            </div>
            <textarea value={props.text.clone()} oninput={props.oninput.clone()}></textarea>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct SelectionProps {}

#[function_component]
pub fn Selection(_props: &SelectionProps) -> Html {
    let (selection, _) = use_store::<state::Selected>();
    if !selection.value.is_empty() {
        let text = format!("selection: {}", &selection.value);
        html! {
            <div class="content">
                <p>{text}</p>
            </div>
        }
    } else {
        html! {}
    }
}
