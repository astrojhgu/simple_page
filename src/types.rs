use std::path::PathBuf;

use handlebars::Handlebars;
use rocket::{
    http::{
        uri::Segments,
        RawStr,
    },
    request::{
        FromSegments,
        FromParam,
    },
};

use std::option::NoneError;

//use rocket::http::{impl_from_uri_param_identity};

pub struct FileWithExt<const EXT: &'static str>(pub PathBuf);

//impl_from_uri_param_identity!([Path] (const EXT:&'static str) FileWithExt<{EXT}>)

pub struct DirPath(pub PathBuf);

pub struct StaticDir(pub PathBuf);

pub struct DataDir(pub PathBuf);

pub struct SpecialFile(pub PathBuf);

pub struct Template<'reg>(pub Handlebars<'reg>);

impl<const EXT: &'static str> FileWithExt<EXT> {
    pub fn show(&self) -> String {
        let mut result = "".to_string();
        result += "File with Extension ";
        result += EXT;
        result += " ";
        result += self.0.to_str().unwrap();
        result
    }

    pub fn upper_level(&self) -> Option<FileWithExt<EXT>> {
        let mut p = self.0.clone();
        if let Some(fname) = self.0.file_name() {
            let x = p.pop() && p.pop();
            if x {
                p.push(PathBuf::from(fname));
                Some(FileWithExt(p))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn current_dir(&self) -> PathBuf {
        if let Some(p) = self.0.parent() {
            PathBuf::from(p)
        } else {
            PathBuf::from("")
        }
    }

    pub fn parent_dir(&self) -> PathBuf {
        if let Some(p) = self.current_dir().as_path().parent() {
            PathBuf::from(p)
        } else {
            PathBuf::from("")
        }
    }
}

impl DirPath {
    pub fn parent_dir(&self) -> PathBuf {
        if let Some(p) = self.0.parent() {
            PathBuf::from(p)
        } else {
            PathBuf::from("")
        }
    }
}

impl<'a, const EXT: &'static str> FromSegments<'a> for FileWithExt<EXT> {
    type Error = std::option::NoneError;

    fn from_segments(segs: Segments<'a>) -> std::result::Result<Self, Self::Error> {
        //let path=PathBuf::from(String::from(segs.0));
        let path = PathBuf::from_segments(segs).unwrap();
        let given_ext = path.extension().map(|s| s.to_str())??;
        if given_ext == EXT {
            Ok(FileWithExt::<EXT>(path))
        } else {
            Err(NoneError)
        }
    }
}

impl<'a> FromSegments<'a> for DirPath {
    type Error = NoneError;

    fn from_segments(segs: Segments<'a>) -> std::result::Result<Self, Self::Error> {
        let path = PathBuf::from_segments(segs).unwrap();
        if path.extension().is_none() {
            Ok(DirPath(path))
        } else {
            Err(NoneError)
        }
    }
}

impl<'a> FromParam<'a> for DirPath {
    type Error = NoneError;

    fn from_param(param: &'a RawStr) -> std::result::Result<Self, Self::Error> {
        let path = PathBuf::from(String::from_param(param).unwrap());
        if path.extension().is_none() {
            Ok(DirPath(path))
        } else {
            Err(NoneError)
        }
    }
}

impl<'a> FromParam<'a> for SpecialFile {
    type Error = NoneError;

    fn from_param(param: &'a RawStr) -> std::result::Result<Self, Self::Error> {
        let fname:&str=match param.as_str(){
            ""|
            "index.html"=>"index.html",
            "favicon.ico"=>"favicon.ico",
            "robots.txt"=>"rockets.txt",
            _=>return Err(NoneError),
        };
        Ok(SpecialFile(PathBuf::from(fname)))
    }
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
