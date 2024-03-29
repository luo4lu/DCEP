FROM rust:alpine as builder

RUN sed -i 's/dl-cdn.alpinelinux.org/mirrors.tuna.tsinghua.edu.cn/g' /etc/apk/repositories \
    && apk add openssl-dev build-base --no-cache

#FROM registry.cn-hongkong.aliyuncs.com/cloud-api/digital_transaction:builder as builder

WORKDIR /workspace

ADD ./ /workspace

RUN cargo build --release

FROM alpine

WORKDIR /workspace

RUN sed -i 's/dl-cdn.alpinelinux.org/mirrors.tuna.tsinghua.edu.cn/g' /etc/apk/repositories \
    && apk add openssl --no-cache \
    && ln -s /lib/ld-musl-x86_64.so.1 /lib/ld64.so.1

COPY --from=builder /workspace/target/release/digital_transaction .

EXPOSE 5000

ENTRYPOINT [ "./digital_transaction" ]