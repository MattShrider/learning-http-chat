# Archived

This project is now archived and will no longer be worked on. The original
goal of the project was to create an HTTP chat server with zero dependencies,
but after reaching the point of serializing and deserializing JSON I felt
scope was too large for a learning project.

A follow-up will be made that allows for some sane dependencies, such as
serde-json and potentially something like axum or another concurrency runtime.

# learning_http_chat

A learning project to implement an Http server from scratch
that will be used to run a very simple chat server.

## Warnings

This project should not be used for anything important, the underlying
TcpListener is likely vulnerable to attacks.

## Usage

Running the project starts an http server at 127.0.0.1:8080. As of now
this server will echo back the body provided to it. It only accepts
messages following HTTP/1.1 protocol.

## Testing

Hammer this thing! The following will create 10,000 requests with 1,000
in parallel at a time.

```
seq 1 10000 | xargs -P1000 -I{} curl http://localhost:8080/{} --data test
```
