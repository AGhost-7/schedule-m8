FROM ubuntu:bionic

COPY ./target/release/schedule-m8 /usr/local/bin/schedule-m8

VOLUME /usr/local/schedule-m8/data

CMD ["/usr/local/bin/schedule-m8"]
