use tiny_http::{Header, Request, Response, Server, Method, StatusCode};
use std::str;
use std::fs::File;

use super::model::*;

fn serve_static_file(request: Request, file_path: &str, content_type: &str) -> Result<(),()>{
    let content_type_header = Header::from_bytes("Content-Type", content_type)
       .expect("Might fail cause I aint no web developer");
    let file = File::open(file_path).map_err(|err|{
       eprintln!("ERROR: couldn't open index file: {err}");
    })?;
    let response = Response::from_file(file).with_header(content_type_header);
    
    request.respond(response).map_err(|err|{
       eprintln!("ERROR: could not serve the static file: {err}");
    })
 }
 
 fn serve_404(request: Request) -> Result<(), ()>{
    request.respond(Response::from_string("404").with_status_code(StatusCode(404))).map_err(|err|{
       eprintln!("ERROR: could not respond to request: {err}");
    })
 }

fn serve_search_api(mut request: Request, tf_index: &IndexTF) -> Result<(),()> {
    let mut buf = Vec::new();
    request.as_reader().read_to_end(&mut buf).map_err(|err| {
        eprintln!("ERROR: Could not read from the reader: {err}");
    })?;
    let body = str::from_utf8(&buf).map_err(|err|{
       eprintln!("ERROR: could not parse the request body as UTF-8 string: {err}");
    })?.chars().collect::<Vec<_>>();

    let result = search(tf_index, &body);

    let json = serde_json::to_string(&result.iter().take(20).collect::<Vec<_>>()).map_err(|err|{
        eprintln!("ERROR: could not convert the search result into json: {err}");
    })?;

    let content_type_header = Header::from_bytes("Content-Type", "application/json").expect("Might fail cause I aint no web developer");
    let response = Response::from_string(&json).with_header(content_type_header);
    request.respond(response).map_err(|err|{
        eprintln!("ERROR: could not serve the request: {err}");
    })
}

 fn serve_request(tf_index: &IndexTF, request: Request) -> Result<(),()> {
    println!("Received requests! method: {:?}, url: {:?}", request.method(), request.url());
    
    match (request.method(), request.url()) {
       (Method::Post, "/api/search") =>{
          serve_search_api(request, tf_index)
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

 pub fn start(address: &str, tf_index: &IndexTF) -> Result<(),()>{
    let server = Server::http(&address).map_err(|err|{
        eprintln!("ERROR: could not start server at{address}: {err}");
    })?;

    println!("INFO: Listening at: http://{address}/");

    for request in server.incoming_requests() {
        serve_request(&tf_index, request).ok();
    }

    eprintln!("ERROR: The socket has shut down!");
    Err(())
 }
 