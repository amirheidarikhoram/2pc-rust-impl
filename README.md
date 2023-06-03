# 2PC Rust Implementation

## What is this about?
This project is a naive implementation of the 2PC protocol in Rust. The goal was to do a distributed transaction with one single coordinator and a few participants.


## Coordinator
The server is constructed of a simple TCP listener, wrapped with `Coordinator`, and an HTTP server. The actual coordinator is shared with API handlers through the app state. Every time a participant, from now on called `element`, connects, its message handling and state management is delegated to `Coordinator`.

In this project, the user can send REST post requests to the HTTP server to create records on the coordinator and all other elements.
Then coordinator first initiates a transaction locally on the database to which it is connected and then calls `execute_transaction` to initiate a distributed transaction for all connected elements.
In this step a `MultiPeerTransaction` is created and `Begin` messages with corresponding `query/command` alongside its `variables` are sent to all elements.

At this point, the coordinator is waiting for `Accept` responses to check if all elements are ready to commit the transaction. If all elements are ready, the coordinator sends a `Commit` message to the elements and waits for their responses. If any of the elements respond with a `Reject` message, the coordinator sends a `Reject` to every element, and the transaction is aborted. At the last step `execute_transaction` rejects with Err variant to indicate that the transaction is aborted and no changes are made to the database, neither locally nor on elements.
If Elements respond back with a `Done` message the state `execute_transaction` resolves with an Ok variant to indicate that the transaction is committed and changes should be made to the database.


## Element
The element is a simple application that wraps a TCP client connected to the coordinator. If a `Begin` message is received, it initiates a transaction on its database, executes it, and if nothing goes wrong sends an `Accept` message to the coordinator.
If a `Commit` message is received, it commits the transaction on its database and sends a `Done` message to the coordinator.
If a `Reject` message is received, it aborts the transaction on its database.

Internally, the element uses `tokio` to spawn threads for receiving messages and state management. A thread is just responsible for receiving messages and putting them in a channel. The other hand is a thread responsible for processing messages and state management. This thread is spawned at the startup of the application and is waiting for messages to be received. If a message is received, it is processed based on the local transaction state, and appropriate action is taken.

## How to run
To run the project you should have `rust` and `cargo` installed on your machine. You can install them from [here](https://www.rust-lang.org/tools/install).
After installing dependencies of both three projects, you can run them with the `cargo run` command. You can also run them with `cargo run --release` to run them in release mode.

First, run the coordinator which will run a TCP listener on port `8080` and a HTTP server on port `500`. The coordinator will try to connect to the `2pc` DB on local Postgres.
Then run elements with a command line argument to specify the db name for each element. For example `element 2pc_e1` will run an element with db name `2pc_e1` which will try to connect to `2pc_e1` db on local Postgres.

Note that you should create tables for `Post` on Postgres before running the project. You can avoid creating the table for a random element for testing purposes. If you do so, you will get a `Reject` from the element while doing a distributed transaction that leads to a 500 error on the coordinator.

## Important Notes
You should be aware that this is a basic implementation of the 2PC protocol and not a production-ready implementation. There are a lot of things that can be improved (if you take a look at the code).

## TODO
- [ ] Add better error handling
- [ ] Make it easy for non-technical people to run the project
- [ ] Refactor and apply todos in the code