FROM rust:1.85.0

RUN rustup component add clippy rustfmt

RUN apt-get update && apt-get install --yes libclang-dev

ARG USER=user
ARG GROUP=$USER
ARG USER_ID=1000
ARG GROUP_ID=$USER_ID

RUN groupadd -g $GROUP_ID $GROUP && useradd -m -u $USER_ID -s /bin/sh -g $GROUP $USER

USER $USER:$GROUP
