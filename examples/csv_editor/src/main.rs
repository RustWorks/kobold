use kobold::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement as Input;

mod csv;
mod state;

use state::{Editing, State, Text};

#[component]
fn Editor() -> impl View {
    stateful(State::mock, |state| {
        let onload = {
            let signal = state.signal();

            move |e: Event<Input>| {
                let file = match e.target().files().and_then(|list| list.get(0)) {
                    Some(file) => file,
                    None => return,
                };

                signal.update(|state| state.name = file.name());

                let signal = signal.clone();

                spawn_local(async move {
                    if let Ok(table) = csv::read_file(file).await {
                        signal.update(move |state| state.table = table);
                    }
                })
            }
        };

        bind! { state:
            let onkeydown = move |event: KeyboardEvent<_>| {
                if matches!(event.key().as_str(), "Esc" | "Escape") {
                    state.editing = Editing::None;

                    Then::Render
                } else {
                    Then::Stop
                }
            };
        }

        view! {
            <input type="file" accept="text/csv" onchange={onload} />
            <h1>{ state.name.fast_diff() }</h1>
            <table {onkeydown}>
                <thead>
                    <tr>
                    {
                        state.columns().map(|col| view! { <Head {col} {state} /> }).list()
                    }
                    </tr>
                </thead>
                <tbody>
                {
                    state.rows().map(move |row| view! {
                        <tr>
                        {
                            state.columns().map(move |col| view! {
                                <Cell {col} {row} {state} />
                            })
                            .list()
                        }
                        </tr>
                    })
                    .list()
                }
                </tbody>
            </table>
        }
    })
}

#[component(auto_branch)]
fn Head(col: usize, state: &Hook<State>) -> impl View + '_ {
    let value = state.source.get_text(&state.columns[col]);

    if state.editing == (Editing::Column { col }) {
        let onchange = state.bind(move |state, e: Event<Input>| {
            state.columns[col] = Text::Owned(e.target().value().into());
            state.editing = Editing::None;
        });

        view! {
            <th.edit>
                { value.fast_diff() }
                <input.edit.edit-head {onchange} value={ value.fast_diff() } />
            </th>
        }
    } else {
        let ondblclick = state.bind(move |s, _| s.editing = Editing::Column { col });

        view! { <th {ondblclick}>{ value.fast_diff() }</th> }
    }
}

#[component(auto_branch)]
fn Cell(col: usize, row: usize, state: &Hook<State>) -> impl View + '_ {
    let value = state.source.get_text(&state.rows[row][col]);

    if state.editing == (Editing::Cell { row, col }) {
        let onchange = state.bind(move |state, e: Event<Input>| {
            state.rows[row][col] = Text::Owned(e.target().value().into());
            state.editing = Editing::None;
        });

        view! {
            <td.edit>
                { value.fast_diff() }
                <input.edit {onchange} value={ value.fast_diff() } />
            </td>
        }
    } else {
        let ondblclick = state.bind(move |s, _| s.editing = Editing::Cell { row, col });

        view! { <td {ondblclick}>{ value.fast_diff() }</td> }
    }
}

fn main() {
    kobold::start(view! {
        <Editor />
    });
}