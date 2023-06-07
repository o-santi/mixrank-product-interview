#![allow(unused_imports)]
// all imports are used, but for some reason
// the compiler reports some of them as unused (i think because they
// are macro related?). specifically, tracing is imported for
// #[component] and IntoAttribute is imported for attributes in
// the view! macro.

use leptos::leptos_dom::tracing;
use leptos::IntoAttribute;
use std::collections::HashSet;
use leptos::{Scope, IntoView, CollectView, Transition, SignalUpdate, ReadSignal, WriteSignal, view, create_signal, component, create_resource, create_local_resource};
use crate::db::{App, Column, get_all_churned, get_all_sdks, get_total_apps_count};
use leptos_router::{Router, Routes, Route};
use leptos_meta::{Stylesheet, Link};

#[component]
pub fn sdk_selector(cx: Scope, selected: ReadSignal<HashSet<Column>>, set_selected: WriteSignal<HashSet<Column>>) -> impl IntoView {
  let all_sdks = create_resource(cx, move || (), |_| get_all_sdks());
  view! {
    cx,
    <div class="sdk-selector">
    <Transition fallback={move || view!{cx, <p><i>"...fetching sdks"</i></p>}}>
    <ul>
    {move ||
     all_sdks
     .read(cx)
     .map(|sdks| {
       if let Ok(sdks) = sdks {
         sdks
           .into_iter()
           .map(|sdk| {
             let sdk_column = Column::Sdk(sdk.clone());
             let onchange = move |_| {
               if selected().contains(&sdk_column) {
                 set_selected.update(|s| {s.remove(&sdk_column);});
               } else {
                 set_selected.update(|s| {s.insert(sdk_column.clone());});
               };
             };
             let class = if selected().contains(&Column::Sdk(sdk.clone())) {
               "sdk selected"
             } else {
               "sdk not-selected"
             };
             view! {
               cx,
               <li>
                 <div class=class on:mousedown=onchange>
                 <p>{sdk.name}</p>
                 </div>
                 </li>
             }
           }).collect_view(cx)
       } else {
         view! {cx, <p>"Error"</p>}.into_view(cx)
       }
     })
    }
    
    </ul>
    </Transition>
    </div>
  }
}
    

#[component]
pub fn usage_matrix(
  cx: Scope,
  selected: ReadSignal<HashSet<Column>>,
  set_show_apps: WriteSignal<Vec<App>>,
  set_sdk_pair: WriteSignal<Option<(Column, Column)>>
) -> impl IntoView {
  let apps_count = create_resource(cx, move || (), |_| get_total_apps_count());
  let table_header = move || {
    std::iter::once(view!{cx, <th>"Sdk"</th>}).chain(selected().iter().map(|sdk| {
      view! {
        cx,
        <th>{if let Column::Sdk(s) = sdk {
          s.name.clone()
        } else {
          "other sdks".into()
        }}</th>
      }
    })).collect_view(cx)
  };
  let rows_resource = create_local_resource(cx, move || selected().iter().cloned().collect(), get_all_churned);
  let table_body = move || {
    let column_map = rows_resource.read(cx);
    let apps_count = apps_count.read(cx);
    match (apps_count, column_map) {
      (Some(Ok(apps_count)), Some(Ok(map))) => {
        let from_selected = selected();
        let to_selected = from_selected.clone();
        from_selected.into_iter().map(|from_sdk| {
          let row_name = match &from_sdk {
            Column::Sdk(s) => s.name.clone(),
            Column::All => "other sdks".to_owned()
          };
          let row_values = to_selected.iter().map(|to_sdk| {
            let used_apps = map.get(&(from_sdk.clone(), to_sdk.clone())).unwrap().clone();
            let (from_clone, to_clone) = (from_sdk.clone(), to_sdk.clone());
            let clone_apps = used_apps.clone();
            let intensity = 255 - ((255 * used_apps.len() as i32) / apps_count);
            let style = format!("background-color: rgb(255, {intensity}, {intensity});");
            let onclick = move |_| {
              set_sdk_pair.update(|s| *s = Some((from_clone.clone(), to_clone.clone())));
              set_show_apps.update(|a| *a = clone_apps.clone());
            };
            view!{cx, <td on:mousedown=onclick style=style>{used_apps.len()}</td>}
          }).collect_view(cx);
          view! {
            cx,
            <tr>
              <td><b>{row_name}</b></td>
            {row_values}
            </tr>
          }
        }).collect_view(cx)
      }
      (Some(Err(e)), _) | (_, Some(Err(e))) => {
        view! {
          cx,
          <p> {e.to_string()} </p>
        }.into_view(cx)
      }
      _ => view!{ cx, }.into_view(cx)
    }
  };
  view! {
    cx,
    <Transition fallback = move || view! {cx, <p><i>"...fetching data"</i></p>} >
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
      </Transition>
  }
}

#[component]
fn show_example_apps(cx: Scope, apps: ReadSignal<Vec<App>>, sdk_pair: ReadSignal<Option<(Column, Column)>>) -> impl IntoView {
  let apps = move ||
    match sdk_pair() {
      Some((from_sdk, to_sdk)) => {
        let apps_list = apps().iter().map(|app| {
          let description = format!("{} {:.2}⭐  ({})", app.name.clone(), app.rating(), app.rating_count());
          view! {
            cx,
            <li>
              <a href=app.company_url.clone()><img src=app.artwork_large_url.clone() width=25 height=25/></a>
              {description}
            </li>
          }
        }).collect_view(cx);
        let title = match (from_sdk, to_sdk) {
          (Column::All, Column::All) => format!("{} apps use any of the sdks.", apps().len()),
          (Column::All, Column::Sdk(to)) => format!("{} apps were using another sdk but now use {}", apps().len(), to.name),
          (Column::Sdk(from), Column::All) => format!("{} apps were using {} but now use another sdk.", apps().len(), from.name),
          (Column::Sdk(from), Column::Sdk(to)) if from == to => format!("{} apps use {}", apps().len(), from.name),
          (Column::Sdk(from), Column::Sdk(to)) => format!("{} apps were using {} but now use {}", apps().len(), from.name, to.name)
        };
        view! {
          cx,
          <p>{title}</p>
            <ul>
          {apps_list}
          </ul>
        }.into_view(cx)
      }
      None => {
        view! {
          cx,
          <p>"Select a row to visualize the apps."</p>
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
    <Link rel="preconnect" href="https://fonts.googleapis.com"/>
    <Link rel="preconnect" href="https://fonts.gstatic.com" {"crossorigin"}/>
    <Link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Roboto+Slab:wght@300&display=swap" />
    <Stylesheet id="competitive-matrix" href="/style.css"/>
    <Router>
      <h1>"Competitive Matrix rendering"</h1>
      <main>
      <Routes>
      <Route path="" view= move |cx| {
        let mut set = HashSet::new();
        set.insert(Column::All);
        let (selected, set_selected) = create_signal(cx, set);
        let (sdk_pair, set_sdk_pair) = create_signal(cx, Option::<(Column, Column)>::None);
        let (show_apps, set_show_apps) = create_signal(cx, Vec::new());
        view! {
          cx,
          <div class="columns">
            <div class="sdk-column">
              <SdkSelector selected=selected set_selected=set_selected />
            </div>
            <div class="matrix-column">
            <UsageMatrix selected=selected set_show_apps=set_show_apps set_sdk_pair=set_sdk_pair/>
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
