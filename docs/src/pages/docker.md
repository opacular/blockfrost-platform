## Docker

### Building & running blockfrost-platform

To build the Docker image containing the project binary, in the root folder of the repository:

```console
docker build -t blockfrost-platform .
```

The Docker image named `blockfrost-platform` is locally available. To run it:

```console
docker run -it --init --rm \
-p 3000:3000 \
-v /home/user/my_node.socket:/var/run/node.socket \
blockfrost-platform --node-socket-path /var/run/node.socket \
--network preview --secret my_secret --reward-address my_reward_address
```

> **_NOTE:_** Make sure your Cardano node socket is attached as a volume, i.e. `-v /home/user/my_node.socket:/var/run/node.socket`.

> **_NOTE:_** If you don't specify an IP address (i.e., `-p 3000:3000` instead of `-p 127.0.0.1:3000:3000`) when publishing a container's ports, Docker publishes the port on all interfaces (address `0.0.0.0`) by default. These ports are externally accessible.

### Building & running blockfrost-platform and required services

The below command will build and run the `blocfrost-platform` binary, along with the Cardano node, using Docker Compose.

In the root folder of the repository:

```console
NETWORK=preview SECRET=my-secret REWARD_ADDRESS=my-reward-address docker compose -p preview up --build -d
docker compose watch # Auto rebuild on changes
```

> **_NOTE:_** If you want to avoid running it in the background, omit the `-d` flag.

> **_NOTE:_** If you want to skip building, omit the `--build` flag.

> **_NOTE:_** Setting `-p preview` to the desired network will let you run on different networks without messing your node db. You can omit it if you plan to run on the same network always.

> **_NOTE:_** You don't need to provide `--node-socket-path` since it is already handled inside `docker-compose.yml`.
