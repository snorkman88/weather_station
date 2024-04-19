# Rust as the base image
FROM rust:1.77-buster as build

# create a new empty shell project
RUN USER=root cargo new --bin axum_server
WORKDIR /axum_server

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
#RUN rm src/*.rs

# copy your source tree
COPY ./src ./src

# build for release
RUN rm ./target/release/deps/axum_*
RUN cargo build

## our final base
FROM rust:1.77-slim-buster

# copy the build artifact from the build stage
COPY --from=build /axum_server/target/debug .

# set the startup command to run your binary
CMD ["./weather_station"]