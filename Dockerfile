FROM rust as build

COPY ./src/ /usr/local/src/schedule-m8/src
COPY ./Cargo.lock /usr/local/src/schedule-m8/Cargo.lock
COPY ./Cargo.toml /usr/local/src/schedule-m8/Cargo.toml

WORKDIR /usr/local/src/schedule-m8

RUN cargo build --release

FROM ubuntu:bionic

COPY --from=build /usr/local/src/schedule-m8/target/release/schedule-m8 /usr/local/bin/schedule-m8

ENV SCHEDULE_M8_DATA_DIR /var/lib/schedule-m8/data

VOLUME /var/lib/schedule-m8/data

CMD ["/usr/local/bin/schedule-m8"]
