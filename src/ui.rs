use std::collections::HashSet;
use leptos::{Scope, IntoView, CollectView, view, create_signal, component, tracing, IntoAttribute, SignalUpdate, ReadSignal, WriteSignal, create_resource, Suspense};
use crate::db::{Sdk, App, get_all_churned, get_all_sdks, get_total_apps_count};
use leptos_router::{Router, Routes, Route};
use leptos_meta::Stylesheet;

#[component]
pub fn sdk_selector(cx: Scope, selected: ReadSignal<HashSet<Sdk>>, set_selected: WriteSignal<HashSet<Sdk>>) -> impl IntoView {
  let all_sdks = create_resource(cx, move || (), |_| get_all_sdks());
  view! {
    cx,
    <Suspense fallback={move || view!{cx, <p><i>"...loading"</i></p>}}>
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
                };
                view! {
                  cx,
                  <li>
                    <input type="checkbox" id={sdk.id} on:change=onchange />
                    <label for={sdk.id}>{sdk.name}</label>
                  </li>
                }
              }).collect_view(cx)
          } else {
            view! {cx, <p>"Error"</p>}.into_view(cx)
          }
        })
    }
    </ul>
    </Suspense>
  }
}
    

#[component]
pub fn usage_matrix(cx: Scope, selected: ReadSignal<HashSet<Sdk>>, set_show_apps: WriteSignal<Vec<App>>, set_sdk_pair: WriteSignal<Option<(Sdk, Sdk)>>) -> impl IntoView {
  let _apps_count = create_resource(cx, move || (), |_| get_total_apps_count());
  let table_header = move || {
    std::iter::once(view!{cx, <th>"Sdk"</th>}).chain(selected().iter().map(|sdk| {
      view! { cx, <th>{sdk.name.clone()}</th>}
    })).collect_view(cx)
  };
  let rows_resource = create_resource(cx, move || selected().iter().cloned().collect(), get_all_churned);
  let table_body = move || {
    let data = rows_resource.read(cx);
    match data {
      Some(Ok(map)) => {
        let from_selected = selected();
        let to_selected = from_selected.clone();
        from_selected.into_iter().map(|from_sdk| {
          let to_sdk_row = to_selected.iter().map(|to_sdk| {
            let used_apps = map.get(&(from_sdk.clone(), to_sdk.clone())).unwrap().clone();
            let (from_clone, to_clone) = (from_sdk.clone(), to_sdk.clone());
            let clone_apps = used_apps.clone();
            let onclick = move |_| {
              set_sdk_pair.update(|s| *s = Some((from_clone.clone(), to_clone.clone())));
              set_show_apps.update(|a| *a = clone_apps.clone());
            };
            view!{cx, <td on:mousedown=onclick>{used_apps.len()}</td>}
          }).collect_view(cx);
          view! {
            cx,
            <tr>
            <td>{from_sdk.name}</td>
            {to_sdk_row}
            </tr>
          }
        }).collect_view(cx)
      }
      Some(Err(e)) => {
        view! {
          cx,
          <p> {e.to_string()} </p>
        }.into_view(cx)
      }
      None => view!{ cx, <p> "Error"</p> }.into_view(cx)
    }
  };
  view! {
    cx,
    <Suspense fallback = move || view! {cx, <p><i>"...loading"</i></p>} >
    <table class="matrix">
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

#[component]
fn show_example_apps(cx: Scope, apps: ReadSignal<Vec<App>>, sdk_pair: ReadSignal<Option<(Sdk, Sdk)>>) -> impl IntoView {
  let apps = move || 
    match sdk_pair() {
      Some((from_sdk, to_sdk)) => {
        let apps_list = apps().iter().map(|app| {
          view! {
            cx,
            <li>
              <a href=app.company_url.clone()><img src=app.artwork_large_url.clone() width=25 height=25/></a> <b>{app.name.clone()}</b>
              </li>
          }
        }).collect_view(cx);
        let title = if from_sdk != to_sdk {
          view! {cx, <p>{apps().len()}" apps were using " {from_sdk.name} " but now use " {to_sdk.name}</p>}
        } else {
          view! {cx, <p> {apps().len()} " apps use " {from_sdk.name} </p>}
        };
        view! {
          cx,
          {title}
          {apps_list}
        }.into_view(cx)
      }
      None => {
        view! {
          cx,
          <pre>"Select a row to visualize the apps."</pre>
        }.into_view(cx)
      }
    };
  view! {
    cx,
    <div class="apps">
    {apps}
    </div>
  }
}

#[component]
pub fn MatrixApp(cx: Scope) -> impl IntoView {
  leptos_meta::provide_meta_context(cx);
  view! {
    cx,
    <Stylesheet id="competitive-matrix" href="/style.css"/>
    <Router>
      <h1>"Competitive Matrix rendering"</h1>
      <main>
      <Routes>
      <Route path="" view= move |cx| {
        let (selected, set_selected) = create_signal(cx, HashSet::new());
        let (sdk_pair, set_sdk_pair) = create_signal(cx, Option::<(Sdk, Sdk)>::None);
        let (show_apps, set_show_apps) = create_signal(cx, Vec::new());
        view! {
          cx,
          <div class="columns">
            <div>
              <SdkSelector selected=selected set_selected=set_selected />
            </div>
            <div>
              <UsageMatrix selected=selected set_show_apps=set_show_apps set_sdk_pair=set_sdk_pair/>
            </div>
            <div>
              <ShowExampleApps apps=show_apps sdk_pair=sdk_pair/>
            </div>
          </div>
        }
        }/>
      </Routes>
      </main>
    </Router>
  }
}
