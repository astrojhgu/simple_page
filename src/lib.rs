#![allow(clippy::let_and_return)]

use pulldown_cmark::{html, Options, Parser};
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
#[macro_use]
extern crate serde_json;
use handlebars::Handlebars;
//use rocket::Route;
use rocket::http::Status;
//use rocket::http::Method;
use rocket::handler::Outcome;
//use rocket::outcome::IntoOutcome;
use rocket::response::{content, NamedFile};
use rocket::Data;
use rocket::Handler;
use rocket::Request;

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

#[derive(Clone)]
pub struct MyHandler {
    base_path: std::path::PathBuf,
    article_prefix: String,
    pub article_template: std::sync::Arc<Handlebars<'static>>,
}

pub fn regulate_link(s: &str) -> String {
    if s == "" {
        "".to_string()
    } else {
        s.to_string() + "/"
    }
}

pub fn search_parent(mut path: PathBuf, mut web_path: PathBuf) -> Option<(PathBuf, PathBuf)> {
    if path.exists() {
        Some((path, web_path))
    } else if let Some(_) = path.as_path().extension() {
        let file_name = PathBuf::from(path.file_name().unwrap());
        if path.pop() && path.pop() && web_path.pop() && web_path.pop() {
            path.push(file_name.clone());
            web_path.push(file_name);
            search_parent(path, web_path)
        } else {
            None
        }
    } else {
        if path.pop() && web_path.pop() {
            search_parent(path, web_path)
        } else {
            None
        }
    }
}

impl MyHandler {
    pub fn new(
        base_path: PathBuf,
        article_template: PathBuf,
        dir_template: PathBuf,
        article_prefix: String,
    ) -> MyHandler {
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
            std::fs::read_to_string(dir_template).unwrap(),
        )
        .unwrap();
        MyHandler {
            base_path,
            article_prefix,
            article_template: std::sync::Arc::new(reg),
        }
    }

    pub fn enumerate_article_items(
        &self,
        path: PathBuf,
        web_path: PathBuf,
    ) -> Option<Vec<ArticleItem>> {
        eprintln!("aa={:?} {:?}", path, web_path);
        if path.is_dir() {
            let mut paths: Vec<_> = std::fs::read_dir(path)
                .unwrap()
                .filter_map(|r| r.ok())
                .collect();

            paths.sort_by_key(|dir| dir.path());
            let result = paths
                .iter()
                .filter(|p| {
                    p.path().is_dir()
                        || p.path().extension() == Some(&std::ffi::OsString::from("md"))
                })
                .map(|p| {
                    if p.path().is_dir() {
                        //ArticleItem::Directory{title:p.path().to_string_lossy().into_owned()}
                        ArticleItem::Directory {
                            title: p.file_name().to_string_lossy().into_owned(),
                            link: "/".to_string()
                                + &self.article_prefix
                                + "/"
                                + &regulate_link(
                                    (web_path.to_str().unwrap().to_string()
                                        + "/"
                                        + p.file_name().to_string_lossy().into_owned().as_str())
                                    .as_str(),
                                ),
                        }
                    } else {
                        ArticleItem::Article {
                            //title: p.file_name().to_string_lossy().into_owned(),
                            title: p.path().file_stem().unwrap().to_string_lossy().into_owned(),
                            link: "/".to_string()
                                + &self.article_prefix
                                + "/"
                                + regulate_link(web_path.to_str().unwrap()).as_str()
                                + "/"
                                + p.file_name().to_string_lossy().into_owned().as_str(),
                        }
                    }
                })
                .collect();
            Some(result)
        } else {
            None
        }
    }
}

fn try_open_file(path: &Path) -> std::io::Result<NamedFile> {
    if path.is_dir() {
        eprintln!("not a file name");
        std::io::Result::Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ))
    } else if let Ok(f) = NamedFile::open(path) {
        Ok(f)
    } else {
        eprintln!("Not found in current path, go to parent");
        let file_name = path.file_name().unwrap();
        let mut pb = PathBuf::from(path);
        if pb.pop() && pb.pop() {
            pb.push(file_name);
            try_open_file(pb.as_path())
        } else {
            std::io::Result::Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "file not found",
            ))
        }
    }
}

fn try_read_to_string(path: &Path) -> std::io::Result<String> {
    if path.is_dir() {
        eprintln!("not a file name");
        std::io::Result::Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ))
    } else if let Ok(s) = read_to_string(path) {
        Ok(s)
    } else {
        eprintln!("Not found in current path, go to parent");
        let file_name = path.file_name().unwrap();
        let mut pb = PathBuf::from(path);
        if pb.pop() && pb.pop() {
            pb.push(file_name);
            try_read_to_string(pb.as_path())
        } else {
            std::io::Result::Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "file not found",
            ))
        }
    }
}

impl Handler for MyHandler {
    fn handle<'r>(&self, request: &'r Request, _data: Data) -> Outcome<'r> {
        let mut article_path = self.base_path.clone();
        println!("{:?}", article_path);
        if let Ok(web_path) = request
            .get_segments::<'r, std::path::PathBuf>(0)
            .unwrap_or_else(|| Ok("".into()))
        {
            article_path.push(web_path.clone());
            println!("article path={:?}", article_path);
            if let Some((article_path, web_path)) = search_parent(article_path, web_path) {
                println!("article path={:?}", article_path);
                let current_path = web_path.clone();
                let current_dir = if article_path.is_dir() {
                    current_path.clone()
                } else {
                    current_path
                        .as_path()
                        .parent()
                        .unwrap_or(PathBuf::from("/").as_path())
                        .to_path_buf()
                };
                let mut current_fs_dir = self.base_path.clone();
                current_fs_dir.push(current_dir.clone());
                println!("current dir={:?}", current_dir);
                println!("current path={:?}", current_path);
                println!("current os path={:?}", current_fs_dir);
                let parent = current_path.parent();
                let parent_link = if let Some(x) = parent {
                    String::new() + "/" + &self.article_prefix + "/" + x.to_str().unwrap()
                } else {
                    String::new()
                };
                println!("web path={:?}", web_path);
                if web_path.to_str().unwrap().ends_with(".md") {
                    if let Result::Ok(s) = try_read_to_string(article_path.as_path()) {
                        let mut options = Options::empty();
                        options.insert(Options::ENABLE_STRIKETHROUGH);
                        options.insert(Options::ENABLE_TABLES);
                        let parser = Parser::new_ext(s.as_str(), options);
                        let mut html_output = String::new();
                        html::push_html(&mut html_output, parser);
                        //println!("{}", html_output);
                        if let Ok(s) = self.article_template.as_ref().render(
                            "article_template",
                            &json!({ "content": html_output , "parent_link": parent_link, "current_dir": current_dir}),
                        ) {
                            //println!("{}", s);
                            Outcome::from(request, content::Html(s))
                        } else {
                            Outcome::Failure(Status::InternalServerError)
                        }
                    } else {
                        Outcome::Failure(Status::NotFound)
                    }
                } else if web_path.to_str().unwrap().ends_with(".html") {
                    if let Ok(f) = try_open_file(article_path.as_path()) {
                        Outcome::from(request, f)
                    } else {
                        Outcome::Failure(Status::NotFound)
                    }
                /*
                if let Ok(f)=NamedFile::open(article_path){
                    Outcome::from(request, f)
                }else{
                    Outcome::Failure(Status::NotFound)
                }*/
                } else if article_path.is_dir() {
                    let titles: Vec<_> = self
                        .enumerate_article_items(article_path, web_path)
                        .unwrap()
                        .into_iter()
                        .map(|p| {
                            json!({"title": p.title(),
                            "link": p.link(),
                            })
                        })
                        .collect();
                    if let Ok(s) = self.article_template.as_ref().render(
                        "dir_template",
                        &json!({ "items": titles , "parent_link": parent_link, "current_dir": current_dir}),
                    ) {
                        //println!("{:?}", s);
                        Outcome::from(request, content::Html(s))
                    } else {
                        Outcome::Failure(Status::NotFound)
                    }
                } else if let Ok(f) = NamedFile::open(article_path) {
                    println!("fdsfsdfadssaf");
                    Outcome::from(request, f)
                } else {
                    Outcome::from(
                        request,
                        content::Html("<html><head></head><boby>1<body></html>"),
                    )
                }
            } else {
                Outcome::from(
                    request,
                    content::Html("<html><head></head><boby>aaa<body></html>"),
                )
            }
        } else {
            Outcome::from(
                request,
                content::Html("<html><head></head><boby>fdsafdsa<body></html>"),
            )
        }

        //Outcome::from(request, "<html><head></head><boby>fdsafdsa<body></html>")
    }
}

#[derive(Clone)]
pub struct StaticFileHandler {
    base_path: std::path::PathBuf,
}

impl StaticFileHandler {
    pub fn new(p: PathBuf) -> StaticFileHandler {
        StaticFileHandler { base_path: p }
    }
}

impl Handler for StaticFileHandler {
    fn handle<'r>(&self, request: &'r Request, _data: Data) -> Outcome<'r> {
        let mut file_path = self.base_path.clone();
        if let Ok(web_path) = request
            .get_segments::<'r, std::path::PathBuf>(0)
            .unwrap_or_else(|| Ok("".into()))
        {
            file_path.push(web_path);
            println!("static: {:?}", file_path);
            if let Ok(f) = NamedFile::open(file_path) {
                Outcome::from(request, f)
            } else {
                Outcome::Failure(Status::NotFound)
            }
        } else {
            Outcome::Failure(Status::NotFound)
        }
    }
}

#[derive(Clone)]
pub struct HardFileHandler {
    file: PathBuf,
}

impl HardFileHandler {
    pub fn new(p: PathBuf) -> HardFileHandler {
        HardFileHandler { file: p }
    }
}

impl Handler for HardFileHandler {
    fn handle<'r>(&self, request: &'r Request, _data: Data) -> Outcome<'r> {
        if let Ok(f) = NamedFile::open(self.file.clone()) {
            Outcome::from(request, f)
        } else {
            Outcome::Failure(Status::NotFound)
        }
    }
}
