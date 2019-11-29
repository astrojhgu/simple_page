extern crate simple_page;
use actix_web::middleware::Logger;
use actix_web::{App, HttpServer};
extern crate log;

fn main() {
    let matches = clap::App::new("server")
        .arg(
            clap::Arg::with_name("addr")
                .required(false)
                .takes_value(true)
                .value_name("ip addr:port")
                .short("a")
                .long("addr")
                .help("ip addr with port"),
        )
        .arg(
            clap::Arg::with_name("data_path")
                .required(true)
                .takes_value(true)
                .value_name("data path")
                .short("d")
                .long("dp")
                .help("data path"),
        )
        .get_matches();

    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let addr = matches.value_of("addr").unwrap_or("127.0.0.1:8000");
    let root_path = matches.value_of("data_path").unwrap().to_string();
    let article_path = root_path.clone() + "/articles";
    let misc_article_path = root_path.clone() + "/misc";

    let article_template = root_path.clone() + "/template/article.html";
    let misc_article_template = root_path.clone() + "/template/misc.html";
    let dir_template = root_path.clone() + "/template/dir.html";
    let static_dir = root_path.clone() + "/static";
    let index_path = root_path.clone() + "/index.html";
    let article_cfg = vec![
        simple_page::DynArticleCfg::new(
            article_path.into(),
            article_template.into(),
            dir_template.clone().into(),
            "articles".into(),
        ),
        simple_page::DynArticleCfg::new(
            misc_article_path.into(),
            misc_article_template.into(),
            dir_template.into(),
            "misc".into(),
        ),
    ];

    let static_cfg = simple_page::StaticCfg::new("static".to_string(), static_dir.into());

    //let cfg_func=simple_page::compose_cfg(cfg);
    HttpServer::new(move || {
        let article_cfg = article_cfg.clone();
        let static_cfg = static_cfg.clone();
        //let cfg=cfg;
        App::new()
            .configure(
                //simple_page::compose_cfg(cfg)
                simple_page::compose_cfg1(article_cfg, vec![static_cfg])
                    .with_static_file("/".into(), index_path.clone().into())
                    .with_static_file("/index.html".into(), index_path.clone().into())
                    .with_static_file(
                        "/favicon.ico".into(),
                        (root_path.clone() + "/favicon.ico").into(),
                    )
                    .with_static_file(
                        "/robots.txt".into(),
                        (root_path.clone() + "/robots.txt").into(),
                    )
                    .cfg_fn,
            )
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
    })
    .bind(addr)
    .unwrap()
    .run()
    .unwrap()
}
