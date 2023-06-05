use std::collections::{HashSet, HashMap};
use leptos::{Scope, IntoView, CollectView, view, create_signal, component, tracing, IntoAttribute, SignalUpdate, ReadSignal, WriteSignal, SignalGet, Signal, create_server_action, create_resource, Suspense};
use crate::db::{Sdk, App, GetAllChurned, get_all_churned, get_all_sdks};
use leptos_router::{Router, Routes, Route};


#[component]
pub fn sdk_selector(cx: Scope, selected: ReadSignal<HashSet<Sdk>>, set_selected: WriteSignal<HashSet<Sdk>>) -> impl IntoView {
  let all_sdks = create_resource(cx, move || (), |_| get_all_sdks());
  view! {
    cx,
    <Suspense fallback={move || view!{cx, <p> "...loading"</p>}}>
    <ul>
    {
      all_sdks
        .read(cx)
        .map(|sdks| {
          if let Ok(sdks) = sdks {
            sdks
              .into_iter()
              .map(|sdk| {
                let cloned_sdk = sdk.clone();
                let onchange = move |_| {
                  if selected().contains(&cloned_sdk) {
                    set_selected.update(|s| {s.remove(&cloned_sdk);});
                  } else {
                    set_selected.update(|s| {s.insert(cloned_sdk.clone());});
                  };
                  leptos::log!("{:?}", selected());
                };
                view! { cx,
                        <li>
                        <input type="checkbox" id={sdk.id} on:change=onchange />
                        <label for={sdk.id}>{sdk.name}</label>
                        </li>
                }
              }).collect_view(cx)
          } else {
            view! {cx, <p> "Error" </p>}.into_view(cx)
          }
        })
    }
    </ul>
      </Suspense>
  }
}
    

#[component]
pub fn usage_matrix(cx: Scope, selected: Signal<HashSet<Sdk>>) -> impl IntoView {
  let table_header = move || {
    std::iter::once(view!{cx, <th>"Sdk"</th>}).chain(selected().iter().map(|sdk| {
      view! { cx, <th>{sdk.name.clone()}</th>}
    })).collect_view(cx)
  };
  let rows_resource = create_resource(cx, move || selected().iter().map(|sdk| sdk.name.clone()).collect(), get_all_churned);
  let table_body = move || {
    let data = rows_resource.read(cx);
    match data {
      Some(rows) => {
        rows.unwrap().iter().map(|(from_sdk, to_rows)| {
          let apps = to_rows.iter()
            .map(|apps| {
              let apps_clone = apps.clone();
              let onhover = move |_| leptos::log!("{:?}", apps_clone);
              view!{cx, <td on:mouseenter=onhover>{apps.len()}</td>}
            })
            .collect_view(cx);
          view! {
            cx,
            <tr>
            <td>{from_sdk}</td>
            {apps}
            </tr>
          }
        }).collect_view(cx)
      }
      None => view!{ cx, <p> "Error"</p> }.into_view(cx)
    }
  };
  view! {
    cx,
    <Suspense fallback = move || view! {cx, <p> "...loading"</p>} >
    <table>
      <thead>
       <tr>
       {table_header}
       </tr>
      </thead>
      <tbody>
        {table_body}
      </tbody>
      </table>
      </Suspense>
  }
}

// #[component]
// fn show_example_apps()

#[component]
pub fn MatrixApp(cx: Scope) -> impl IntoView {
  view! {
    cx,
    <Router>
      <header>
      <h1> "Competitive Matrix rendering"</h1>
      </header>
      <main>
      <Routes>
      <Route path="" view= move |cx| {
        let (selected, set_selected) = create_signal(cx, HashSet::new());
        view! {
          cx,
          <div>
            <SdkSelector selected=selected set_selected=set_selected />
            <UsageMatrix selected=Signal::derive(cx, selected) />
          </div>
        }
        }/>
      </Routes>
      </main>
    </Router>
  }
}
