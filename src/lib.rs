#![allow(clippy::let_and_return)]

use actix_files::NamedFile;
use actix_web::Either;
use actix_web::{http, web, HttpResponse};
use pulldown_cmark::{html, Options, Parser};
use std::fs::read_to_string;
use std::path::PathBuf;
#[macro_use]
extern crate serde_json;
use handlebars::Handlebars;

pub struct CfgChain<F>
where
    F: FnOnce(&mut web::ServiceConfig),
{
    pub cfg_fn: Box<F>,
}

impl<F> CfgChain<F>
where
    F: FnOnce(&mut web::ServiceConfig),
{
    pub fn new(f: Box<F>) -> CfgChain<F> {
        CfgChain { cfg_fn: f }
    }

    pub fn add_cfg<H>(self, f: Box<H>) -> CfgChain<impl FnOnce(&mut web::ServiceConfig)>
    where
        H: FnOnce(&mut web::ServiceConfig),
    {
        CfgChain {
            cfg_fn: Box::new(move |cfg: &mut web::ServiceConfig| {
                (self.cfg_fn)(cfg);
                f(cfg);
            }),
        }
    }

    pub fn with_static_file(
        self,
        web_path: String,
        file_path: std::path::PathBuf,
    ) -> CfgChain<impl FnOnce(&mut web::ServiceConfig)> {
        let f: Box<_> = Box::new(move |s: &mut web::ServiceConfig| {
            s.route(
                web_path.clone().as_str(),
                web::get().to(move || NamedFile::open(file_path.clone())),
            );
        });
        self.add_cfg(f)
    }
}

#[derive(Clone)]
pub struct DynArticleCfg {
    pub article_path: PathBuf,
    //pub article_template: String,
    pub article_template: std::sync::Arc<Handlebars>,
    pub article_prefix: String,
}

#[derive(Debug)]
pub enum ArticleItem {
    Directory { title: String, link: String },
    Article { title: String, link: String },
}

impl ArticleItem {
    pub fn title(&self) -> String {
        match self {
            ArticleItem::Directory { ref title, .. } => title.clone(),
            ArticleItem::Article { ref title, .. } => title.clone(),
        }
    }

    pub fn link(&self) -> String {
        match self {
            ArticleItem::Directory { ref link, .. } => link.clone(),
            ArticleItem::Article { ref link, .. } => link.clone(),
        }
    }
}

impl DynArticleCfg {
    pub fn new(
        article_path: PathBuf,
        article_template: PathBuf,
        dir_template: PathBuf,
        article_prefix: String,
    ) -> DynArticleCfg {
        let mut reg = Handlebars::new();
        reg.unregister_escape_fn();
        reg.register_escape_fn(handlebars::no_escape);
        reg.register_template_string(
            "article_template",
            std::fs::read_to_string(article_template).unwrap(),
        )
        .unwrap();

        reg.register_template_string(
            "dir_template",
            std::fs::read_to_string(dir_template).unwrap(),
        )
        .unwrap();

        DynArticleCfg {
            article_path,
            //article_template: "./data/template".into(),
            article_template: std::sync::Arc::new(reg),
            article_prefix,
        }
    }
}

#[derive(Clone)]
pub struct StaticCfg {
    pub static_file_dir: PathBuf,
    pub static_prefix: String,
}

impl StaticCfg {
    pub fn new(static_prefix: String, static_file_dir: PathBuf) -> StaticCfg {
        StaticCfg {
            static_file_dir,
            static_prefix,
        }
    }
}

pub fn regulate_link(s: &str) -> String {
    if s == "" {
        "".to_string()
    } else {
        s.to_string() + "/"
    }
}

pub fn enumerate_article_items(
    cfg: &DynArticleCfg,
    path: PathBuf,
    web_path: String,
) -> Vec<ArticleItem> {
    //println!("web path={}", web_path);
    if path.is_dir() {
        let mut paths: Vec<_> = std::fs::read_dir(path)
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        paths.sort_by_key(|dir| dir.path());
        paths
            .iter()
            .filter(|p| {
                p.path().is_dir() || p.path().extension() == Some(&std::ffi::OsString::from("md"))
            })
            .map(|p| {
                if p.path().is_dir() {
                    //ArticleItem::Directory{title:p.path().to_string_lossy().into_owned()}
                    ArticleItem::Directory {
                        title: p.file_name().to_string_lossy().into_owned(),
                        link: "/".to_string()
                            + cfg.article_prefix.as_str()
                            + "/"
                            + regulate_link(web_path.as_str()).as_str()
                            + p.file_name().to_string_lossy().into_owned().as_str(),
                    }
                } else {
                    ArticleItem::Article {
                        //title: p.file_name().to_string_lossy().into_owned(),
                        title: p.path().file_stem().unwrap().to_string_lossy().into_owned(),
                        link: "/".to_string()
                            + cfg.article_prefix.as_str()
                            + "/"
                            + regulate_link(web_path.as_str()).as_str()
                            + p.file_name().to_string_lossy().into_owned().as_str(),
                    }
                }
            })
            .collect()
    } else {
        Vec::new()
    }
}

pub fn static_handler(cfg: &StaticCfg, path: web::Path<String>) -> Option<NamedFile> {
    let mut file_path = cfg.static_file_dir.clone();
    let path = path.into_inner();
    file_path.push(path);

    if let Ok(f) = NamedFile::open(file_path) {
        Some(f)
    } else {
        None
    }
}

pub fn article_handler(
    cfg: &DynArticleCfg,
    path: web::Path<String>,
) -> Either<HttpResponse, NamedFile> {
    let mut article_path = cfg.article_path.clone();
    let path = path.into_inner();
    article_path.push(path.clone());
    //println!("{:?}", article_path.is_dir());
    //println!("{:?}", path.ends_with("md"));
    let current_path = PathBuf::from(path.clone());
    let parent = current_path.parent();
    let parent_link = if let Some(x) = parent {
        String::new() + "/" + &cfg.article_prefix.clone() + "/" + x.to_str().unwrap()
    } else {
        String::new()
    };
    println!("p={}", parent_link);
    if path.ends_with(".md") {
        if let Result::Ok(s) = read_to_string(article_path) {
            let mut options = Options::empty();
            options.insert(Options::ENABLE_STRIKETHROUGH);
            options.insert(Options::ENABLE_TABLES);
            let parser = Parser::new_ext(s.as_str(), options);
            let mut html_output = String::new();
            html::push_html(&mut html_output, parser);
            //println!("{}", html_output);
            if let Ok(s) = cfg.article_template.as_ref().render(
                "article_template",
                &json!({ "content": html_output , "parent_link": parent_link}),
            ) {
                //println!("{}", s);
                Either::A(
                    HttpResponse::Ok()
                        .set_header(http::header::CONTENT_TYPE, "text/html")
                        .body(s),
                )
            } else {
                Either::A(HttpResponse::NotFound().body("not found"))
            }
        } else {
            Either::A(HttpResponse::NotFound().body("not found"))
        }
    } else if article_path.is_dir() {
        //println!("parent={:?}",parent_link);
        let titles: Vec<_> = enumerate_article_items(cfg, article_path, path)
            .into_iter()
            .map(|p| {
                json!({"title": p.title(),
                "link": p.link(),
                })
            })
            .collect();
        if let Ok(s) = cfg.article_template.as_ref().render(
            "dir_template",
            &json!({ "items": titles , "parent_link": parent_link}),
        ) {
            //println!("{:?}", s);
            Either::A(
                HttpResponse::Ok()
                    .set_header(http::header::CONTENT_TYPE, "text/html")
                    .body(s),
            )
        } else {
            Either::A(HttpResponse::NotFound().body("not found"))
        }
    } else if let Ok(f) = NamedFile::open(article_path) {
        Either::B(f)
    } else {
        Either::A(HttpResponse::NotFound().body("not found"))
    }

    //println!("{:?}", article_path);
}

pub fn cfg_server(
    article_cfgs: Vec<DynArticleCfg>,
    static_cfgs: Vec<StaticCfg>,
    srv_cfg: &mut web::ServiceConfig,
) {
    for ac in article_cfgs {
        let cfg1 = ac.clone();
        srv_cfg.route(
            (ac.article_prefix.clone() + "/{tail:.*}").as_str(),
            web::get().to(move |x: web::Path<String>| article_handler(&cfg1, x)),
        );
    }

    for sc in static_cfgs {
        let cfg1 = sc.clone();
        srv_cfg.route(
            (sc.static_prefix.clone() + "/{tail:.*}").as_str(),
            web::get().to(move |x: web::Path<String>| static_handler(&cfg1, x)),
        );
    }
}

pub fn compose_cfg(
    article_cfg: Vec<DynArticleCfg>,
    static_cfg: Vec<StaticCfg>,
) -> impl FnOnce(&mut web::ServiceConfig) {
    move |srv_cfg: &mut web::ServiceConfig| {
        cfg_server(article_cfg, static_cfg, srv_cfg);
    }
}

pub fn compose_cfg1(
    article_cfg: Vec<DynArticleCfg>,
    static_cfg: Vec<StaticCfg>,
) -> CfgChain<impl FnOnce(&mut web::ServiceConfig)> {
    CfgChain::new(Box::new(move |srv_cfg: &mut web::ServiceConfig| {
        cfg_server(article_cfg, static_cfg, srv_cfg)
    }))
}
