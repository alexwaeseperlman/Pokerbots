# Pokerzero, compete with poker algorithms

To get it running, first start install diesel, run the database, and run the migrations.
```
docker-compose up -d
cargo install diesel_cli --no-default-features --features postgres 
cd website
diesel migration run
```

First create a `.env` file in website that looks like this:
```
AZURE_SECRET="your-azure-secret"
DATABASE_URL="postgres://postgres:postgres@localhost:5432/postgres"
SECRET_KEY="very secret key just (this must be at least 64 characters) aaaaaaaaaaaaaaajk"
MICROSOFT_CLIENT_ID = "your-client-id"
REDIRECT_URI = "http://localhost:3000/api/login"
MICROSOFT_TENANT_ID = "your-tenant-id"

PFP_S3_BUCKET="s3-bucket-name-for-pfps"
```

Then to run the website you need to build the frontend and start the server. In one shell run
```
cd website
cargo run
```
And in another run
```
cd website/app
npm install
npm run build
```
Then you will have pokerzero running at `http://localhost:3000`.

Also we use S3 to store profile pictures and execute games. In order to use these
features you will need to have a valid aws configuration in your `~/.aws/` folder.
Follow [https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-files.html](this guide)
for help setting that up.

You should give your IAM user full access to S3, Batch, RDS and Fargate. The server
ensures your resources have the correct configuration on startup. We are not
responsible for accidental costs.