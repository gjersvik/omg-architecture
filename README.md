# OMG Agents
An actors inspired library without its own runtime.

## The problem

I often end up having to use one or more frameworks in my projects. Tokio + Bevy. And getting two or more runtime to work together in single application or system is annoying. Channels can only go so far.

The biggest problem code can run in different modes.
* Async often used by networking and web liberties often built on tokio.
* Sync the default used by rust sdk. Often used in cli applications and tools. Easy world to be in.
* Non-blocking often used by games and other real time applications. 

And there modes are poisonous to etch other and have to live in different treads. And i find myself reimplementing common tings like message passing/networking and state management over and over again.

## Goals

Create an architecture pattern as library. Where i can write environment agnostics code. That can easily run inside one or more run-times. In the same application or over the network. 

Right now i have 3 tings i want to abstract over:
1. State-management. Be able to persist state and having it just be there when i restart the application.
2. Networking. Message passing between tasks, treads and computers.
3. Runtime agnostics business logic. Using my own take on the actor pattern.

## Architecture
The man tool for runtime agnostic code is channels and treads. Channels allows me to send messages between different contexts. And treads allows me to escape the current context. (Except in web browsers.)

On the top is Agents. Agents process messages and have a typed Input and Output message type. So for every message it can modify its own state or generate 0 or more output messages. But Agents do not part of a runtime so some external code need to tell the agent when to process messages. Also however drives the agent can get an immutable reference to its internal state.

Agents can listen to channels and you give it at mapping function on subscribe. Can also publish output messages to one or more channels using a mapping function. 

Agents can be put into an Agency. Allows multiple agents to be run in framework native way or background treads. It is not possible to get handle to internal state of any agents that exists in the runtime. There is no way to know when an agent in an agency changes it state. But it is possible to remove and inject and agent from an agency.

There is a special form of channel called a log. Where a channel deletes its messages after everyone have read it. Logs persists messages using a event store service. This allows new subscribes to read all messages not only the ones that happened after they subscribed.

Some agents are persisted agents that can store its state using a log. The log is replayed to the agent at creation.

Services exist to connect to external code. They are represented as two channels that forms a Input output pair. If to services implements the same type of io channel they are equivalent. In this way the core code can be runtime agnostic.

## Todo list for Minimal Viable Toolkit
1. Create the preferred syntax for the sync demo. With a fake toolbox.
1. Sync demo using a sync only core with sqlite backend.
1. Async demo with async supported framework and backend.
1. Non-blocking demo with non-blocking supported framework and backend.
