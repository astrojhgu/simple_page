use rocket::http::Method;
use rocket::Route;
//use rocket::handler::Outcome;
//use rocket::outcome::IntoOutcome;
//use rocket::Request;
//use rocket::Data;
//use rocket::Handler;
//use rocket::response::content;
extern crate simple_page;

use simple_page::{HardFileHandler, MyHandler, StaticFileHandler};

fn main() {
    let article_handler = MyHandler::new(
        std::path::PathBuf::from("./sample_data/articles"),
        "./sample_data/template/article.html".into(),
        "./sample_data/template/dir.html".into(),
        "articles".into(),
    );

    let misc_handler = MyHandler::new(
        std::path::PathBuf::from("./sample_data/misc"),
        "./sample_data/template/misc.html".into(),
        "./sample_data/template/dir.html".into(),
        "misc".into(),
    );

    let static_handler = StaticFileHandler::new("./sample_data/static".into());

    let index_handler = HardFileHandler::new("./sample_data/index.html".into());
    let robots_handler = HardFileHandler::new("./sample_data/robots.txt".into());
    let favicon_handler = HardFileHandler::new("./sample_data/favicon.ico".into());

    rocket::ignite()
        .mount(
            "/articles",
            vec![
                Route::new(Method::Get, "/<a..>", article_handler.clone()),
                Route::new(Method::Get, "/", article_handler),
            ],
        )
        .mount(
            "/misc",
            vec![
                Route::new(Method::Get, "/<a..>", misc_handler.clone()),
                Route::new(Method::Get, "/", misc_handler),
            ],
        )
        .mount(
            "/",
            vec![
                Route::new(Method::Get, "/", index_handler.clone()),
                Route::new(Method::Get, "/index.html", index_handler),
                Route::new(Method::Get, "/robots.txt", robots_handler),
                Route::new(Method::Get, "/favicon.ico", favicon_handler),
            ],
        )
        .mount(
            "/static",
            vec![Route::new(Method::Get, "/<a..>", static_handler)],
        )
        .launch();
}
