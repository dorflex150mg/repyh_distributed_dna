# Server API

This Rust application implements a simple server API using **Actix-web**  to store DNA sequences in a distributed database.

### Configuration
Configuration file must like this:
   
   ```json
   [
       ["server_ip", "api_ip"],
       ["peer1", "peer2"]
   ]

### Major Dependencies

- **Actix-web**: A powerful, pragmatic, and extremely fast web framework for Rust.
- **Tokio**: An asynchronous runtime for Rust, used for handling asynchronous I/O and networking.
- **Serde**: A framework for serializing and deserializing Rust data structures efficiently and generically.
- **SQLite (via `rusqlite`)**: A lightweight, serverless, self-contained SQL database engine, used for local data storage in the application.

## Running the Server

Ensure that you have the required configuration and dependencies set up. Once everything is in place, you can run the server using the following command:

    ```bash
    bash run_cluster

To run a single instance, use the following command:

    ```bash
    DATABASE="var/dna0.db" FILENAME="conf/ips0.json" cargo run 

## Running Test

To run the test, use the following command:
    ```bash
    bash tester.sh 

