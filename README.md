# Compete with poker algorithms

## Running locally
Run `scripts/setup-env`. It will request any necessary variables from you. Then you can start
all the services using `docker compose up`, or run the services you want by using 
`docker compose up sqs db s3 createbuckets`, and running each service you want
manually using `cargo run`.

## Deploying to AWS
Coming soon...