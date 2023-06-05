use cfg_if::cfg_if;
pub mod db;
pub mod ui;

cfg_if! {
  if #[cfg(feature = "hydrate")] {
    use leptos::*;
    use wasm_bindgen::prelude::wasm_bindgen;
    use crate::ui::MatrixApp;
    
    #[wasm_bindgen]
    pub fn hydrate() {
      leptos::mount_to_body(|cx| {
        view!{cx, <MatrixApp/>}
      });
    }
  }
}
