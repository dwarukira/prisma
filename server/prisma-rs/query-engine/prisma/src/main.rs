mod context;
mod error;
mod req_handlers;
mod schema;
mod json;

use actix_web::{fs, http::Method, server, App, HttpRequest, Json, Responder};
use context::PrismaContext;
use error::PrismaError;
use req_handlers::{GraphQlBody, GraphQlRequestHandler, PrismaRequest, RequestHandler};
use serde_json;
use std::env;
use std::sync::Arc;

pub type PrismaResult<T> = Result<T, PrismaError>;

struct HttpHandler {
    context: PrismaContext,
    graphql_request_handler: GraphQlRequestHandler,
}

#[allow(unused_variables)]
fn main() {
    let http_handler = HttpHandler {
        context: PrismaContext::new(),
        graphql_request_handler: GraphQlRequestHandler,
    };
    let http_handler_arc = Arc::new(http_handler);

    env::set_var("RUST_LOG", "actix_web=debug");
    env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let sys = actix::System::new("prisma");
    let address = "127.0.0.1:8000";

    server::new(move || {
        App::with_state(Arc::clone(&http_handler_arc))
            .resource("/", |r| {
                r.method(Method::GET).with(playground);
                r.method(Method::POST).with(handler);
            })
            .resource("/datamodel", |r| r.method(Method::GET).with(data_model_handler))
    })
    .bind(address)
    .unwrap()
    .start();

    println!("Started http server: {}", address);
    let _ = sys.run();
}

fn handler((json, req): (Json<Option<GraphQlBody>>, HttpRequest<Arc<HttpHandler>>)) -> impl Responder {
    let http_handler = req.state();
    let req: PrismaRequest<GraphQlBody> = PrismaRequest {
        body: json.clone().unwrap(),
        path: req.path().into(),
        headers: req
            .headers()
            .iter()
            .map(|(k, v)| (format!("{}", k), v.to_str().unwrap().into()))
            .collect(),
    };

    let result = http_handler.graphql_request_handler.handle(req, &http_handler.context);
    serde_json::to_string(&result)
}

fn data_model_handler<T>(_: HttpRequest<T>) -> impl Responder {
    schema::load_datamodel_file().unwrap()
}

fn playground<T>(_: HttpRequest<T>) -> impl Responder {
    fs::NamedFile::open("playground.html")
}
