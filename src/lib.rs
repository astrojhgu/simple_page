#![allow(clippy::let_and_return)]

use pulldown_cmark::{html, Options, Parser};
use std::fs::read_to_string;
use std::path::PathBuf;
#[macro_use]
extern crate serde_json;
use handlebars::Handlebars;
use rocket::Route;
use rocket::http::Status;
use rocket::http::Method;
use rocket::handler::Outcome;
use rocket::outcome::IntoOutcome;
use rocket::Request;
use rocket::Data;
use rocket::Handler;
use rocket::response::{content, NamedFile};

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
pub struct MyHandler{
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


impl MyHandler{
    pub fn new(base_path: PathBuf, 
        article_template: PathBuf,
        dir_template: PathBuf, 
        article_prefix: String,
    )->MyHandler{
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
        MyHandler{
            base_path,
            article_prefix,
            article_template: std::sync::Arc::new(reg)
        }
    }

    pub fn enumerate_article_items(&self, path: PathBuf, web_path: PathBuf)->Vec<ArticleItem>{
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
                                + &self.article_prefix
                                + "/"
                                + &regulate_link((web_path.to_str().unwrap().to_string()
                                + p.file_name().to_string_lossy().into_owned().as_str()).as_str()),
                        }
                    } else {
                        ArticleItem::Article {
                            //title: p.file_name().to_string_lossy().into_owned(),
                            title: p.path().file_stem().unwrap().to_string_lossy().into_owned(),
                            link: "/".to_string()
                                + &self.article_prefix
                                + "/"
                                + regulate_link(web_path.to_str().unwrap()).as_str()
                                + p.file_name().to_string_lossy().into_owned().as_str(),
                        }
                    }
                })
                .collect()
        } else {
            Vec::new()
        }
    }
}


impl Handler for MyHandler{
    fn handle<'r>(&self, request: &'r Request, data: Data)->Outcome<'r>{
        let mut article_path = self.base_path.clone();
        println!("{:?}", article_path);
        if let Ok(web_path)=request.get_segments::<'r, std::path::PathBuf>(0).unwrap_or(Ok("".into())) {
            article_path.push(web_path.clone());
            println!("article path={:?}", article_path);
            let current_path = PathBuf::from(web_path.clone());
            let parent = current_path.parent();
            let parent_link = if let Some(x) = parent {
                String::new() + "/" + &self.article_prefix + "/" + x.to_str().unwrap()
            } else {
                String::new()
            };
            println!("web path={:?}", web_path);
            if web_path.to_str().unwrap().ends_with(".md"){
                if let Result::Ok(s) = read_to_string(article_path) {
                    let mut options = Options::empty();
                    options.insert(Options::ENABLE_STRIKETHROUGH);
                    options.insert(Options::ENABLE_TABLES);
                    let parser = Parser::new_ext(s.as_str(), options);
                    let mut html_output = String::new();
                    html::push_html(&mut html_output, parser);
                    //println!("{}", html_output);
                    if let Ok(s) = self.article_template.as_ref().render(
                        "article_template",
                        &json!({ "content": html_output , "parent_link": parent_link}),
                    ) {
                        //println!("{}", s);
                        Outcome::from(request, content::Html(s))
                    } else {
                        Outcome::Failure(Status::InternalServerError)
                    }
                } else {
                    Outcome::Failure(Status::NotFound)
                }
            }else if(article_path.is_dir()){
                let titles: Vec<_> = self.enumerate_article_items(article_path, web_path)
                .into_iter()
                .map(|p| {
                    json!({"title": p.title(),
                    "link": p.link(),
                    })
                })
                .collect();
                if let Ok(s) = self.article_template.as_ref().render(
                    "dir_template",
                    &json!({ "items": titles , "parent_link": parent_link}),
                ) {
                    //println!("{:?}", s);
                    Outcome::from(request, content::Html(s))
                } else {
                    Outcome::Failure(Status::NotFound)
                }
            }else if let Ok(f)=NamedFile::open(article_path){
                println!("fdsfsdfadssaf");
                Outcome::from(request, f)
            }
            else{
                Outcome::from(request, content::Html ("<html><head></head><boby>1<body></html>"))
            }
        }
        else{
            Outcome::from(request, content::Html ("<html><head></head><boby>fdsafdsa<body></html>"))
        }

        //Outcome::from(request, "<html><head></head><boby>fdsafdsa<body></html>")
        
        
    }
}

#[derive(Clone)]
pub struct StaticFileHandler{
    base_path: std::path::PathBuf,   
}

impl StaticFileHandler{
    pub fn new(p: PathBuf)->StaticFileHandler{
        StaticFileHandler{
            base_path: p
        }
    }
}

impl Handler for StaticFileHandler{
    fn handle<'r>(&self, request: &'r Request, data: Data)->Outcome<'r>{
        let mut file_path=self.base_path.clone();
        if let Ok(web_path)=request.get_segments::<'r, std::path::PathBuf>(0).unwrap_or(Ok("".into())){
            file_path.push(web_path);
            println!("static: {:?}", file_path);
            if let Ok(f)=NamedFile::open(file_path){
                Outcome::from(request, f)
            }else{
            Outcome::Failure(Status::NotFound)           
        }
        }else{
            Outcome::Failure(Status::NotFound)           
        }
    }
}

#[derive(Clone)]
pub struct HardFileHandler{
    file: PathBuf
}

impl HardFileHandler{
    pub fn new(p: PathBuf)->HardFileHandler{
        HardFileHandler{
            file: p
        }
    }
}

impl Handler for HardFileHandler{
    fn handle<'r>(&self, request: &'r Request, data: Data)->Outcome<'r>{
        if let Ok(f)=NamedFile::open(self.file.clone()){
            Outcome::from(request, f)
        }else{
            Outcome::Failure(Status::NotFound)
        }
    }
}
