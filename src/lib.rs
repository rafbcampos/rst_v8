use actix_web::{dev::Server, get, post, web, App, HttpResponse, HttpServer, Responder};
use rusty_v8 as v8;
use serde::Deserialize;
use std::time::Duration;
use tokio::time::timeout;

#[get("/health/full")]
async fn health_full() -> impl Responder {
    HttpResponse::Ok().body("Ok")
}

#[derive(Deserialize)]
struct EvaluateReqBody {
    code: String,
}

#[post("/evaluate")]
async fn evaluate(data: web::Json<EvaluateReqBody>) -> impl Responder {
    let code = data.into_inner().code;

    match timeout(
        Duration::from_secs(10),
        tokio::task::spawn_blocking(move || {
            let isolate = &mut v8::Isolate::new(v8::CreateParams::default());
            let handle_scope = &mut v8::HandleScope::new(isolate);
            let context = v8::Context::new(handle_scope);
            let scope = &mut v8::ContextScope::new(handle_scope, context);

            let code = v8::String::new(scope, &code).unwrap();
            let script = v8::Script::compile(scope, code, None).unwrap();
            let result = script.run(scope).unwrap();

            result.to_string(scope).unwrap().to_rust_string_lossy(scope)
        }),
    )
    .await
    {
        Ok(Ok(result)) => HttpResponse::Ok().body(result),
        Ok(Err(_)) => HttpResponse::InternalServerError().body("Evaluation failed"),
        Err(_) => HttpResponse::RequestTimeout().body("Evaluation timed out"),
    }
}

pub fn run(address: &str) -> Result<Server, std::io::Error> {
    // Initialize V8
    let platform = v8::new_default_platform(0, false).make_shared();
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();

    let server = HttpServer::new(|| App::new().service(health_full).service(evaluate))
        .bind(address)?
        .run();

    Ok(server)
}
