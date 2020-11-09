#![feature(proc_macro_hygiene, decl_macro, try_trait)]
#[macro_use]
extern crate rocket;

use handlebars::Handlebars;

use rocket::{fairing::AdHoc, response::NamedFile, State};

use std::path::PathBuf;

extern crate simple_page;

use simple_page::types::{DataDir, StaticDir, Template};

#[get("/index.html", rank = 0)]
pub fn home_page(data_dir: State<DataDir>) -> Option<NamedFile> {
    NamedFile::open(data_dir.0.join("index.html")).ok()
}

fn main() {
    rocket::ignite()
        .mount(
            "/",
            routes![
                simple_page::static_file,
                simple_page::html_handler,
                simple_page::not_found,
                simple_page::markerdown_handler,
                simple_page::dir_handler,
                simple_page::root_handler,
                home_page
            ],
        )
        .attach(AdHoc::on_attach("Static Dir", |rocket| {
            let static_dir = rocket.config().get_str("static_dir").unwrap().to_string();

            let data_dir = rocket.config().get_str("data_dir").unwrap().to_string();

            let article_template = rocket
                .config()
                .get_str("article_template")
                .unwrap()
                .to_string();

            let dir_template = rocket.config().get_str("dir_template").unwrap().to_string();

            let mut reg = Handlebars::new();
            reg.unregister_escape_fn();
            reg.register_escape_fn(handlebars::no_escape);
            reg.register_template_string(
                "article_template",
                std::fs::read_to_string(article_template.clone()).unwrap_or_else(|_| {
                    eprintln!("{:?} not found", article_template);
                    panic!()
                }),
            )
            .unwrap();

            reg.register_template_string(
                "dir_template",
                std::fs::read_to_string(dir_template.clone()).unwrap_or_else(|_| {
                    eprintln!("{:?} not found", dir_template);
                    panic!()
                }),
            )
            .unwrap();

            let rkt = rocket
                .manage(StaticDir(PathBuf::from(static_dir)))
                .manage(DataDir(PathBuf::from(data_dir)))
                .manage(Template(reg));
            Ok(rkt)
        }))
        .launch();
}
