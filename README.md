# Compete with poker algorithms

## Running locally
Run `scripts/setup-env`. It will request any necessary variables from you. Then you can start
all the services using `docker compose up`, or run the services you want by using 
`docker compose up sqs db s3 createbuckets`, and running each service you want
manually using `cargo run`.

### Dependencies
- You should ensure that rust is installed:
  ```
	curl https://sh.rustup.rs -sSf | bash -s -- -y
	echo 'source $HOME/.cargo/env' >> $HOME/.bashrc
	```
- Node should be installed:
  ```
	curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.3/install.sh | bash
  # In a new shell
	nvm install 20
	nvm use 20
	```
- openssl and postgres
  On linux:
	```
	sudo apt install libssl-dev libpq-dev
	```
	On mac:
	```
	brew install openssl postgresql
	```

## Deploying to AWS
Coming soon...
