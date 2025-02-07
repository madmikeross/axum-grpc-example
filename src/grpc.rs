use tonic::{Request, Response, Status};
use hello::greeter_server::{Greeter, GreeterServer};
use hello::{HelloRequest, HelloResponse};

pub mod hello {
    tonic::include_proto!("hello");
}

#[derive(Default)]
pub struct MyGreeter;

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(&self, req: Request<HelloRequest>) -> Result<Response<HelloResponse>, Status> {
        let name = req.into_inner().name;
        let reply = HelloResponse {
            message: format!("Hello {}!", name),
        };
        Ok(Response::new(reply))
    }
}

pub fn grpc_router() -> GreeterServer<MyGreeter> {
    GreeterServer::new(MyGreeter::default())
}