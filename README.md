# Server API

This Rust application implements a simple server API using **Actix-web**  to store DNA sequences in a distributed database.
The database is composed by 5 nodes running process-local databases. Once one of them receives a DNA sequence post requests, it broadcasts it to the other nodes. Once it receives a byzantine majority of acks, it responds to the client.  

### Configuration
Configuration file must like this:
   
    ```json
    [
    	[
    		"127.0.0.1:9090",
    		"127.0.0.1:8080"
    	],
    
    	[
    		"127.0.0.1:8081",
    		"127.0.0.1:8082",
    		"127.0.0.1:8083",
    		"127.0.0.1:8084"
    	]
    ]
    ```

## Running the Server

Ensure that you have the required configuration and dependencies set up. Once everything is in place, you can run the server using the following command:

    ```bash
    bash run_cluster
    ```

To run a single instance, use the following command:

    ```bash
    DATABASE="var/dna0.db" FILENAME="conf/ips0.json" cargo run 
    ```

## Running Test

To run the test, use the following command:
    ```bash
    bash tester.sh 
    ``
#Post Release

On the Post Release branch, a more complete version is available, with a dedicated client. This client does the same things that the tester script does:

    ```bash
    cd client
    cargo run
    ```
This version is more readable, has correct and more complete signature checking and has better performance, especially with large DNA sequences.

##TODO
* The ability to rollback incomplete updates, i.e., transactional logic to deal with non-byzantine majority operations. 
* Monotonic counters per client to avoid delayed update attacks. 
* More generic sender module using reflection.
