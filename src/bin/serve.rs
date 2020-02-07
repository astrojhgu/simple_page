use rocket::Route;
use rocket::http::Method;
extern crate simple_page;

use simple_page::{MyHandler, StaticFileHandler, HardFileHandler};

fn main() {
    let matches = clap::App::new("server")
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

    let data_path:String=matches.value_of("data_path").unwrap().to_string();

    let article_handler=MyHandler::new((data_path.clone()+"/articles").into(), (data_path.clone()+"/template/article.html").into(), (data_path.clone()+"/template/dir.html").into(), "articles".into());

    let misc_handler=MyHandler::new((data_path.clone()+"/misc").into(), (data_path.clone()+"/template/misc.html").into(), (data_path.clone()+"/template/dir.html").into(), "misc".into());


    let static_handler=StaticFileHandler::new((data_path.clone()+"/static").into());

    let index_handler=HardFileHandler::new((data_path.clone()+"/index.html").into());
    let robots_handler=HardFileHandler::new((data_path.clone()+"/robots.txt").into());
    let favicon_handler=HardFileHandler::new((data_path+"/favicon.ico").into());

    rocket::ignite().mount("/articles", vec![Route::new(Method::Get, "/<a..>", article_handler.clone()), Route::new(Method::Get, "/", article_handler)])
    .mount("/misc", vec![Route::new(Method::Get, "/<a..>", misc_handler.clone()), Route::new(Method::Get, "/", misc_handler)])
    .mount("/", vec![Route::new(Method::Get, "/", index_handler.clone()), Route::new(Method::Get, "/index.html", index_handler),
    Route::new(Method::Get, "/robots.txt", robots_handler),
    Route::new(Method::Get, "/favicon.ico", favicon_handler),
    ])
    .mount("/static", vec![Route::new(Method::Get, "/<a..>", static_handler)])
    .launch();    
}
