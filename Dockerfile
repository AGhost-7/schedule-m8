FROM ubuntu:trusty

COPY ./target/release/schedule-m8 /usr/local/bin/schedule-m8

CMD ["/usr/local/bin/schedule-m8"]
