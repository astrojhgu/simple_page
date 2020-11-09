#![feature(proc_macro_hygiene, decl_macro, try_trait)]
#[macro_use]
extern crate rocket;

use clap::{App, Arg};
use handlebars::Handlebars;

use rocket::{fairing::AdHoc, response::NamedFile, State};

use std::path::PathBuf;

extern crate simple_page;

use simple_page::types::{DataDir, StaticDir, Template};



fn main() {
    let matches=App::new("serve")
    .arg(
        Arg::new("root")
        .short('r')
        .long("root")
        .takes_value(true)
        .required(true)
        .value_name("root dir")
    ).get_matches();
    
    let root_dir=matches.value_of("root").unwrap().to_string();

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
                simple_page::top_level,
                simple_page::index,
            ],
        )
        .attach(AdHoc::on_attach("Static Dir", move |rocket| {
            let static_dir = root_dir.clone()+"/"+rocket.config().get_str("static_dir").unwrap();

            let data_dir = root_dir.clone();

            let article_template = root_dir.clone()+"/"+rocket
                .config()
                .get_str("article_template")
                .unwrap();

            let dir_template = root_dir.clone()+"/"+rocket.config().get_str("dir_template").unwrap();

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
