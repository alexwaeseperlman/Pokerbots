# Gameplay
The gameplay worker runs games.

## Config vars
- `BOT_RUNTIME_MEMORY_LIMIT` the maximum amount of memory used by a bot while running
- `SQS_ADDRESS` the address of the sqs server
- `COMPILED_BOT_S3_BUCKET` the name of the s3 bucket that compiled bots are uploaded to
- `GAME_LOGS_S3_BUCKET` the name of the s3 bucket that game logs are uploaded to
- `NEW_GAMES_QUEUE_URL` the url of the sqs queue that new games are read from
- `GAME_RESULTS_QUEUE_URL` the url of the sqs queue that game results are sent to

## Supported packages for running bots
Right now we only install the following python packages: `numpy pandas scipy scikit-learn`.