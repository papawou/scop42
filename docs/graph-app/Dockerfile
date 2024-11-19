FROM alpine:latest AS builder

RUN apk add\
    nodejs\
    npm

WORKDIR /node/app

ENTRYPOINT [ "bash"]

FROM builder AS starter

COPY /tools/start.sh /tmp
RUN chmod 777 /tmp/start.sh

ENTRYPOINT ["/tmp/start.sh"]