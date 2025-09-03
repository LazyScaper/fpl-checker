FROM rust:trixie AS build
LABEL authors="LazyScaper"

WORKDIR /app
COPY . .
RUN cargo build --release

FROM rust:trixie

COPY --from=build /app/target/release/fpl_checker /usr/local/bin/fpl_checker

RUN chmod +x /usr/local/bin/fpl_checker

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8000

EXPOSE 8000

ENTRYPOINT ["/usr/local/bin/fpl_checker", "--api"]