use stylist::yew::styled_component;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;
use yewdux::prelude::*;

use crate::state;
use crate::N_SIM;

#[derive(Properties, Debug, PartialEq)]
pub struct DismissablePanelProps {
    pub dismiss: Callback<MouseEvent>,
    #[prop_or_default]
    pub children: Html,
}

#[styled_component]
pub fn DismissablePanel(props: &DismissablePanelProps) -> Html {
    html! {
        <div class={css!("display: flex; background-color: lightgray; flex-direction: column;")}>
            <div class={css!("display: flex; flex-flow: row-reverse;")}>
                <DismissButton onclick={props.dismiss.clone()} />
                <div class={css!("flex: 1;")} />
            </div>
            {props.children.clone()}
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct SimulationPanelProps {}

#[styled_component]
pub fn SimulationPanel(_props: &SimulationPanelProps) -> Html {
    /*
            <div>
                <button onclick={props.simulate.clone()}>
                    {format!("run simulation {N_SIM} times")}
                </button>
                <SimulationResults results={props.results.clone()} />
            </div>
    */
    html! {
        <div>
            <div><h2>{"Simulation of Next Choice"}</h2></div>
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
                <div class={bar_style}>{"|"}</div>
                <div class={css!("background-color: whitesmoke; flex: 1;")}></div>
            </div>
            </td>
        </tr>
    }
}

#[derive(Properties, PartialEq)]
pub struct SimulationResultsProps {
    pub results: Option<Vec<(String, u64)>>,
}

#[styled_component]
pub fn SimulationResults(props: &SimulationResultsProps) -> Html {
    if let Some(results) = &props.results {
        html! {
            <table>
                <tr><th>{"candidate"}</th><th width={"80%"}>{"selection count"}</th></tr>
                {
                    results.into_iter().map(|(candidate, count)| {
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
pub struct HistoryHalflifeProps {
    pub oninput: Callback<InputEvent>,
    pub value: f64,
}

#[styled_component]
pub fn HistoryHalflife(props: &HistoryHalflifeProps) -> Html {
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
pub struct HistoryPanelProps {}

#[styled_component]
pub fn HistoryPanel(props: &HistoryPanelProps) -> Html {
    let (history, history_dispatch) = use_store::<state::History>();
    let oninput = history_dispatch.reduce_mut_callback_with(|history, e: InputEvent| {
        let input: HtmlTextAreaElement = e.target_unchecked_into::<HtmlTextAreaElement>();
        history.value = from_lines(&input.value());
    });
    let content = history.value.join("\n");
    html! {
        <Text heading={"history"} text={content} {oninput} />
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
pub fn CandidatesPanel(props: &CandidatesPanelProps) -> Html {
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
pub struct DismissButtonProps {
    pub onclick: Callback<MouseEvent>,
}

#[styled_component]
pub fn DismissButton(props: &DismissButtonProps) -> Html {
    html! {
        <button
            class={css!("color: red; justify-self: right; align-self: start; height: 1.6rem;")}
            onclick={props.onclick.clone()}
        >{"X"}</button>
    }
}

#[derive(Properties, PartialEq)]
pub struct ModeSelectProps {
    pub buttons: Html,
}

#[styled_component]
pub fn ModeSelect(props: &ModeSelectProps) -> Html {
    html! {
        <div class={css!("width: 50%; padding: 3rem; margin: 1rem;")}>
            { props.buttons.clone() }
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
        <div class={css!("width: 80%; margin: 3rem; height: 80%;")}>
            <h3>{props.heading.clone()}</h3>
            <textarea value={props.text.clone()} oninput={props.oninput.clone()}></textarea>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct SelectionProps {
    pub text: Option<String>,
}

#[function_component]
pub fn Selection(props: &SelectionProps) -> Html {
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
