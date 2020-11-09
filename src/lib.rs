#![allow(clippy::let_and_return)]
#![allow(incomplete_features)]
#![feature(proc_macro_hygiene, decl_macro, const_generics, try_trait)]
#[macro_use]
extern crate rocket;

use pulldown_cmark::{html, Options, Parser};

use std::path::PathBuf;
#[macro_use]
extern crate serde_json;

//use rocket::Route;

//use rocket::http::Method;

//use rocket::outcome::IntoOutcome;

use rocket::{
    http::Status,
    response::{content, NamedFile, Redirect},
    uri, State,
};

pub mod types;
use types::{ArticleItem, DataDir, DirPath, FileWithExt, Template, SpecialFile};

pub fn regulate_link(s: &str) -> String {
    if s == "" {
        "".to_string()
    } else {
        s.to_string() + "/"
    }
}

pub fn enumerate_article_items(data_dir: PathBuf, web_path: String) -> Option<Vec<ArticleItem>> {
    let path = data_dir.join(web_path.clone());
    eprintln!("path={:?}", path);
    if path.is_dir() {
        let mut paths: Vec<_> = std::fs::read_dir(path)
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        paths.sort_by_key(|dir| dir.path());
        let result = paths
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
                            + &regulate_link(
                                (web_path.clone()
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
                            + regulate_link(web_path.as_str()).as_str()
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

#[get("/<root>/<path..>", rank = 10)]
pub fn static_file(root: String, path: PathBuf, data_dir: State<DataDir>) -> Option<NamedFile> {
    NamedFile::open(data_dir.0.join(PathBuf::from(root)).join(path)).ok()
}

#[get("/<root>/<path..>", rank = 4)]
pub fn dir_handler(
    root: String,
    path: DirPath,
    data_dir: State<DataDir>,
    template: State<Template>,
) -> Option<content::Html<String>> {
    let data_dir = data_dir.0.clone();
    eprintln!("aaa={:?}", path.0);
    let web_path = root.clone() + "/" + path.0.to_string_lossy().to_owned().to_string().as_str();
    if let Some(items) = enumerate_article_items(data_dir, web_path) {
        let titles: Vec<_> = items
            .iter()
            .map(|p| {
                json!({"title": p.title(),
                "link": p.link(),
                })
            })
            .collect();

        let parent_link = "/".to_string()
            + root.as_str()
            + "/"
            + path
                .parent_dir()
                .to_string_lossy()
                .to_owned()
                .to_string()
                .as_str();

        let current_dir = "/".to_string()
        + root.as_str()
        + "/"
        + path.0
            .to_string_lossy()
            .to_owned()
            .to_string()
            .as_str();
        eprintln!("cd1={:?}", path.parent_dir());
        eprintln!("path={:?}", path.0);

        if let Ok(s) = template.0.render(
            "dir_template",
            &json!({ "items": titles , "parent_link": parent_link, "current_dir": current_dir}),
        ) {
            //println!("{:?}", s);
            Some(content::Html(s))
        } else {
            eprintln!("a");
            None
        }
    } else {
        eprintln!("b");
        None
    }
}


#[get("/<fname>", rank=5)]
pub fn top_level(fname: SpecialFile, data_dir: State<DataDir>) -> Option<NamedFile> {
    eprintln!("{:?}", fname.0);
    
    eprintln!("{:?}", data_dir.0.join(fname.0.clone()));
    NamedFile::open(data_dir.0.join(fname.0)).ok()
}

#[get("/", rank=5)]
pub fn index(data_dir: State<DataDir>) -> Option<NamedFile> {
    NamedFile::open(data_dir.0.join("index.html")).ok()
}


#[get("/<root>", rank = 20)]
pub fn root_handler(
    root: String,
    data_dir: State<DataDir>,
    template: State<Template>,
) -> Option<content::Html<String>> {
    dir_handler(root, DirPath(PathBuf::from("")), data_dir, template)
}

#[get("/<root>/<path..>", rank = 3)]
pub fn markerdown_handler(
    root: String,
    path: FileWithExt<"md">,
    data_dir: State<DataDir>,
    template: State<Template>,
) -> Option<content::Html<String>> {
    let file_path = data_dir.0.as_path().join(root.clone()).join(path.0.clone());
    //file_path.to_str().unwrap().to_string()
    if let Result::Ok(s) = std::fs::read_to_string(file_path.as_path()) {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        let parser = Parser::new_ext(s.as_str(), options);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        //println!("{}", html_output);
        eprintln!("cd={:?}", path.current_dir());
        let parent_link =
            "/".to_string() + root.as_str() + "/" + path.current_dir().to_str().unwrap();
        if let Ok(s) = template.0.render(
            "article_template",
            &json!({ "content": html_output , "parent_link": parent_link, "current_dir": parent_link}),
        ) {
            Some(content::Html(s))
        } else {
            None
        }
    } else {
        None
    }
}

#[get("/404", rank=100)]
pub fn not_found() -> Status {
    Status::NotFound
}


#[get("/<root>/<path..>", rank = 2)]
pub fn html_handler(
    root: String,
    path: FileWithExt<"html">,
    data_dir: State<DataDir>,
) -> Result<NamedFile, Redirect> {
    if let Ok(f) = NamedFile::open(
        data_dir
            .0
            .as_path()
            .join(PathBuf::from(root.clone()).join(path.0.clone())),
    ) {
        Ok(f)
    } else if let Some(p) = path.upper_level() {
        eprintln!("{:?} {:?}", p.0, path.0);
        let u = "/".to_string() + root.as_str() + "/" + p.0.to_str().unwrap();
        eprintln!("{}", u);
        Err(Redirect::to(u))
    } else {
        eprintln!("Not found");
        Err(Redirect::to(uri!(not_found)))
    }
}
