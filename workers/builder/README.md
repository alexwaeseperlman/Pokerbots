# Builder

Download newly uploaded bots, build them, package an executable, and upload them to s3.


## Required config vars
- `UPLOADED_BOT_S3_BUCKET` the name of the s3 bucket that bots are uploaded to.
- `AMQP_ADDRESS` the address of the amqp server