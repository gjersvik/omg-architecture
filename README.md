# OMG Architecture
A set of libraries that implements my personal preferences.

## The problem

I often end up having to use one or more frameworks in my projects. Tokio + Bevy. And getting two or more runtime to work together in single application or system is annoying. Channels can only go so far.

The biggest problem code can run in different modes.
* Async often used by networking and web liberties often built on tokio.
* Sync the default used by rust sdk. Often used in cli applications and tools. Easy world to be in.
* Non-blocking often used by games and other real time applications. 

And there modes are poisonous to etch other and have to live in different treads. And i find myself reimplementing common tings like message passing/networking and state management over and over again.

## Goals

Create a toolbox set of tools that can work in any context and enable communication between contexts. 

Have kafka-esk topic where one or more publisher can post messages to a topic. And one or more readers can read from a true the history using a local cursor. Topics comes in 3 main types:
* At most once: Maximum throughput, minium latency, no durability. Set how many messages to remember default 1. No history can only read the latest message.
* At lest once: Good balance between speed and durability. Can set the minium time a message should be durable default 1 week. 
* Exactly once: When only durability matters. No settings all messages are stored until the topic as a hole is deleted.  

Save state using event sourcing on exactly once topic.

Both reader and writer have async, sync and non-blocking api.

## Architecture
I like actor frameworks but don't like that they are frameworks. So i will work around a concept i call agent. As in i need to call my agent. Agents can publish and subscribe to topics. And they can have there own typed state. 

The top level struck/object is an Agency that is injected with backends that enables features. The agency can live as global variable in a project. From the user side the Agency is where you get agents from.

The users of Agents and Agency should not need to know about the backends. So the backend capabilities will be injected in a type erased fashion. As in using trait objects and not generics.

The Agency also keeps common state for all agents. Will in most cases be a singleton but multiple can coexist say when testing. 

## Packages
This project is separated out into different packages for mostly to enable working in different environments with different requirements. 

### Core
The omg_core package with contain the agent and agency struct. And all the traits needed by backend crates. Will also contain simple mock backends for used in testing and as null backends. 

Will have minimal dependencies.

### Sqlite Storage backend.
The omg_sqlite package will implement the storage trait using sqlite.

### Three demo/example apps.
* async_demo Todo app running as an async web app server.
* sync_demo Todo app running as cli application.
* non_blocking_demo Todo app running as egui app.

## Todo list
1. Sync demo using a sync only core with a memory only backend.
1. Sync only version of sqlite backend.
1. Async demo with async supported framework and backend.
1. Non-blocking demo with non-blocking supported framework and backend.
