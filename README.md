# axum-grpc-example
A lightweight example of using Axum with gRPC.

### Run the service
Create your environment file by copying the example
```shell
cp .env.example .env
```
Make sure all containers are removed
```shell
docker compose down
```
Remove all volumes as well
```shell
docker compose down -v
```
Build and bring up the containers in the bockground
```shell
docker compose up -build -d
```
Check the logs to see if they started
```shell
docker compose logs
```
Check the health endpoint:
```shell
curl http://localhost:3000/health
```
Check the gRPC endpoint with any gRPC client, we suggest [evans](https://github.com/ktr0731/evans):
```shell
evans --proto proto/counter.proto repl
package counter
service CounterService
call IncrementCounter
```
You should see a response like:
```shell
{
  "value": 1
}
```

### Making a database change
Create a migration in the `migrations` directory, similar to `001_create_counter.sql`:
```sql
CREATE TABLE IF NOT EXISTS counter
(
    id    SERIAL PRIMARY KEY,
    count INTEGER NOT NULL DEFAULT 0
);

-- Insert a default record
INSERT INTO counter (count)
SELECT 0
WHERE NOT EXISTS (SELECT 1 FROM counter);
```
Commit your change and restart the service container, when it starts it will run migrations
and update the database.

Alternatively, you can manually apply the migration. When the postgres container is built, it saves a copy of the
migrations to a volume, which means you can manually restart the db container and run the migrations without needing
sqlx. This is particularly helpful if you have a new query you need to prepare before the service can build. If you want
to run the migrations on the database directly, restart just the postgres container first
```shell
docker compose up -d db
```
Now you can connect and run the new migration
```shell
docker compose exec db psql -U myuser -d mydatabase -f migrations/*.sql
```

For any new queries against your table that use the `sqlx::query!` macro, you need to run `cargo sqlx prepare` locally
before docker's build phase so that queries can be statically checked for correctness. First, make sure you set
a `DATABASE_URL` environment variable:
```shell
export DATABASE_URL=postgres://myuser:mypassword@localhost:5432/mydatabase
```
Notice this url is pointing at localhost, which if you are running a docker compose environment, will be the host of the
postgres container. Now run prepare
```shell
cargo sqlx prepare
```
This will generate any missing entries for new queries in the `.sqlx` directory. Make sure to commit this change. Now
when `cargo build --release` runs in the build phase, it can verify your new query's correctness and build correctly.
```shell
docker compose up --build axum-grpc-service -d
```