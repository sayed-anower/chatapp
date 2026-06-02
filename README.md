# 🚀 Chat Server Powered By FR-RUST 

A high-performance, horizontally scalable real-time chat application built in Rust. Powered by the `fr-rust` framework, this system is designed to efficiently handle thousands of concurrent WebSocket connections across multiple distributed servers.

## ✨ Features

* **Real-time Messaging:** Low-latency, bi-directional communication using WebSockets.
* **Distributed Architecture:** Fully supports running multiple server instances. Node-to-node message broadcasting is synchronized via Redis Pub/Sub.
* **Fearless Scalability:** Leverages Rust's memory safety and concurrency models to handle heavy workloads with a minimal resource footprint.
* **Stateless API Design:** Easily deployable behind standard load balancers (e.g., Nginx, HAProxy, AWS ALB).

## 🏗️ Architecture & Multi-Server Support

To support horizontal scaling, the application relies on a **Message Broker (Redis)** to route messages between different server instances. 

**How it works:**
1. A client connected to **Node A** sends a message intended for a client on **Node B**.
2. **Node A** processes the message and publishes it to a centralized Redis channel.
3. **Node B** (along with all other active nodes) is subscribed to this channel and receives the broadcast.
4. **Node B** identifies that the recipient is connected to its instance and pushes the message down their specific WebSocket connection.