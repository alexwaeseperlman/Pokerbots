# Compete with poker algorithms

## Running locally
Run `scripts/setup-env`. It will request any necessary variables from you. Then you can start
all the services using `docker compose up`, or run `docker compose -f dev-compose.yml up`
to watch for changes and recompile if necessary.

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

## Developing
When you're working on a service, you probably don't want to rebuild and run the docker 
containers every time you make a change. This is when the dev-compose file is useful.

- Always make sure that you're running the builder and gameplay workers in the same environment.
  Otherwise they might run into issues.  A more permanent solution to this would be to combine them into one service, but that's not a top priority.