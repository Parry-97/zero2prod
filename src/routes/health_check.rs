use actix_web::{HttpRequest, HttpResponse, Responder};

pub async fn health_check(_req: HttpRequest) -> impl Responder {
    //NOTE: `HttpResponseBuilder` implements Responder as well - we could therefore omit our
    //call to `finish` and shorten our handler to:
    HttpResponse::Ok()
}
