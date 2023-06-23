# Shared
Structs and methods that all backend services share.

## Config vars
- `S3_ADDRESS` the endpoint url for an AWS S3 compatible server.
- `SQS_ADDRESS`the endpoint url for an AWS SQS compatible server. 
When these aren't defined we default to using the AWS credentials configured
in your environment.