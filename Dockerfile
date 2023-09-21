FROM rust:latest as builder

RUN apt-get update -y \
    && apt-get install -y libprotobuf-dev protobuf-compiler cmake

RUN USER=root cargo new --bin eat-where-la-backend
WORKDIR ./eat-where-la-backend

COPY ./Cargo.toml ./
RUN cargo build --release

COPY ./src ./src
RUN rm ./target/release/deps/eat_where_la_backend
RUN cargo build --release

FROM debian:bullseye-slim
ARG APP=/usr/src/app

RUN apt-get update \
    && apt-get upgrade -y \
    && apt-get install -y ca-certificate \
    && rm -rf /var/lib/apt/lists/*

ENV APP_USER=appuser
RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

COPY --from=builder /eat-where-la-backend/target/release/eat-where-la-backend ${APP}/eat-where-la-backend

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
EXPOSE 3000

WORKDIR ${APP}
CMD ["./eat-where-la-backend"]