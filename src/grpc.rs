use counter::counter_service_server::{CounterService, CounterServiceServer};
use counter::{CounterValue, Empty};
use sqlx::PgPool;
use tonic::{Request, Response, Status};

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