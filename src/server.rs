use tiny_http::{Header, Request, Response, Server, Method, StatusCode};
use core::str;
use std::fs::File;

use std::io;

use super::model::*;

fn serve_404(request: Request) -> io::Result<()>{
   request.respond(Response::from_string("404").with_status_code(StatusCode(404)))
}

fn serve_500(request: Request) -> io::Result<()>{
  request.respond(Response::from_string("500").with_status_code(StatusCode(500)))
}

fn serve_400(request:Request, message: &str) -> io::Result<()>{
  request.respond(Response::from_string(format!("400: {message}")).with_status_code(StatusCode(400)))
}

fn serve_static_file(request: Request, file_path: &str, content_type: &str) -> io::Result<()>{
   let content_type_header = Header::from_bytes("Content-Type", content_type).expect("Might fail cause I aint no web developer");
   let file = match File::open(file_path){
      Ok(file) => file,
      Err(err) => {
         eprintln!("ERROR: could not serve file {file_path}: {err}");
         if err.kind() == io::ErrorKind::NotFound {
            return serve_404(request);
         }
         return serve_500(request);
      }
   };
   request.respond(Response::from_file(file).with_header(content_type_header))
 }
 

fn serve_search_api(mut request: Request, model: &Model) -> io::Result<()> {
   let mut buf = Vec::new();
   if let Err(err) = request.as_reader().read_to_end(&mut buf){
      eprintln!("ERROR: Could not read from the reader: {err}");
      return serve_500(request);
   }
   let body = match str::from_utf8(&buf) {
      Ok(body) => body.chars().collect::<Vec<_>>(),
      Err(err) => {
         eprintln!("ERROR: could not interpret body as UTF-8 string: {err}");
         return serve_400(request, "Body must be a valid UTF-8 string");
      }
   };

   let result = search(model, &body);

   let json = match serde_json::to_string(&result.iter().take(20).collect::<Vec<_>>()){
      Ok(json) => json,
      Err(err) =>{
         eprintln!("ERROR: could not convert the search result into json: {err}");
         return serve_500(request)
      }
   };

   let content_type_header = Header::from_bytes("Content-Type", "application/json").expect("Might fail cause I aint no web developer");
   request.respond(Response::from_string(&json).with_header(content_type_header))
}

 fn serve_request(model: &Model, request: Request) -> io::Result<()> {
   println!("Received requests! method: {:?}, url: {:?}", request.method(), request.url());
   
   match (request.method(), request.url()) {
      (Method::Post, "/api/search") =>{
         serve_search_api(request, model)
      }

      (Method::Get, "/") | (Method::Get, "/index.html") => {
      serve_static_file(request, "index.html", "text/html; charset=utf-8")
      }

      (Method::Get, "/index.js") =>{
      serve_static_file(request, "index.js", "text/javascript; charset=utf-8")
      }

      _ => {
         serve_404(request)
      }
   }
}

 pub fn start(address: &str, model: &Model) -> Result<(),()>{
   let server = Server::http(&address).map_err(|err|{
      eprintln!("ERROR: could not start server at{address}: {err}");
   })?;

   println!("INFO: Listening at: http://{address}/");

   for request in server.incoming_requests() {
      serve_request(&model, request).map_err(|err|{
      eprintln!("ERROR: could not serve the response: {err}");
      }).ok(); // <- Keep the network open 
   }

   eprintln!("ERROR: The socket has shut down!");
   Err(())
}
 