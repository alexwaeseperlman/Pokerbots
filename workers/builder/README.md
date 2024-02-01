# Builder

Download newly uploaded bots, build them, package an executable, and upload them to s3.


## Config vars
- `BOT_S3_BUCKET` the name of the s3 bucket that bots are uploaded to.
- `COMPILED_BOT_S3_BUCKET` the name of the s3 bucket that the compiled bots are uploaded to.
- `SQS_ADDRESS` the address of the sqs server. If blank then use your aws credentials from env
- `BUILD_LOGS_S3_BUCKET` the name of the s3 bucket that the build logs are uploaded to.
- `BUILD_RESULTS_QUEUE_URL` the url of the sqs queue that the build results are sent to.