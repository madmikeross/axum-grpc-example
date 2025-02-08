#!/bin/sh

echo "Starting database migrations"
sqlx migrate run
echo "Finished database migrations"

echo "Starting service"
exec /app/axum-grpc-example
