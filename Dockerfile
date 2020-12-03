FROM rust:alpine as build

WORKDIR /usr/src/app
RUN apk add musl-dev

COPY . .

RUN cargo build --release

FROM scratch

COPY --from=build /usr/src/app/target/release/web .

EXPOSE 8080

CMD ["./web"]
