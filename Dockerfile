FROM rust:alpine as builder
RUN apk add --no-cache musl-dev curl git libjxl-tools imagemagick libwebp-tools
WORKDIR /usr/src/holligerme
COPY . .
RUN mkdir /build && cargo install --path . --root /build

FROM alpine
EXPOSE 3000
WORKDIR /app
COPY --from=builder /build/bin .
COPY ./assets assets
COPY ./content content
ENTRYPOINT [ "./holligerme" ]