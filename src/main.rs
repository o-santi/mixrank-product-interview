use actix_files::Files;
use actix_web::{HttpServer, App, Responder, get};
use leptos::{get_configuration, view};
use leptos_actix::{generate_route_list, LeptosRoutes};
use product_eng_interview::ui::MatrixApp;
use cfg_if::cfg_if;
// use product_eng_interview::db::{create_connection, get_churned, SqlResult};
// use leptos_actix::{generate_route_list, LeptosRoutes};


cfg_if! {
  if #[cfg(feature="ssr")] {

    #[get("/style.css")]
    async fn css() -> impl Responder {
      actix_files::NamedFile::open_async("./style.css").await
    }
    
    #[actix_web::main]
    async fn main() -> std::io::Result<()> {
      let conf = get_configuration(None).await.unwrap();
      
      let addr = conf.leptos_options.site_addr;

      product_eng_interview::db::register_server_functions();
      
      // Generate the list of routes in your Leptos App
      let routes = generate_route_list(|cx| view! { cx, <MatrixApp/>});

      HttpServer::new(move || {
        let leptos_options = &conf.leptos_options;
        let site_root = &leptos_options.site_root;
        App::new()
          .service(css)
          .route("/api/{tail:.*}", leptos_actix::handle_server_fns())
          .leptos_routes(leptos_options.to_owned(), routes.to_owned(),
                         |cx| view! { cx, <MatrixApp/>})
          .service(Files::new("/", site_root))
      }).bind(addr)
        .expect("could not bind address")
        .run()
        .await
    }
  } else {
    fn main() {
      // intentionally left empty
      // as this is client side main function.
    }
  }
}
