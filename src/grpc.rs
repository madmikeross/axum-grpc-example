use counter::counter_service_server::{CounterService, CounterServiceServer};
use counter::{CounterValue, Empty};
use hello::greeter_server::{Greeter, GreeterServer};
use hello::{HelloRequest, HelloResponse};
use sqlx::PgPool;
use tonic::{Request, Response, Status};

pub mod hello {
    tonic::include_proto!("hello");
}

pub mod counter {
    tonic::include_proto!("counter");
}

#[derive(Clone)]
pub struct MyCounterService {
    db: PgPool,
}

#[tonic::async_trait]
impl CounterService for MyCounterService {
    async fn increment_counter(&self, _: Request<Empty>) -> Result<Response<CounterValue>, Status> {
        // Directly query the PgPool (no transaction needed)
        let row = sqlx::query!("UPDATE counter SET count = count + 1 RETURNING count")
            .fetch_one(&self.db) // Use the pool directly
            .await
            .map_err(|e| Status::internal(format!("DB Error: {}", e)))?;

        Ok(Response::new(CounterValue { value: row.count }))
    }
}

pub async fn counter_service(db: PgPool) -> CounterServiceServer<MyCounterService> {
    CounterServiceServer::new(MyCounterService { db })
}

#[derive(Default)]
pub struct MyGreeter;

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        req: Request<HelloRequest>,
    ) -> Result<Response<HelloResponse>, Status> {
        Ok(Response::new(HelloResponse {
            message: format!("Hello {}!", req.into_inner().name),
        }))
    }
}

pub async fn greeter_service() -> GreeterServer<MyGreeter> {
    GreeterServer::new(MyGreeter::default())
}
