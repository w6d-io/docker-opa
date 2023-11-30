FROM rust:1.73-bullseye AS build

ENV GOLANG_VERSION 1.21
ARG JOB_TOKEN
ARG JOB_USER
ENV CARGO_NET_GIT_FETCH_WITH_CLI true
# gcc for cgo
RUN apt-get update && apt-get install -y --no-install-recommends \
		g++ \
		gcc \
		clang \
		libc6-dev \
		make \
		pkg-config \
		wget \
		tar \
	&& rm -rf /var/lib/apt/lists/*

RUN set -eux; \
\
	wget https://go.dev/dl/go1.21.linux-amd64.tar.gz; \
	tar -xvf go1.21.linux-amd64.tar.gz; \
	mv go /usr/local

ENV GOPATH /go
ENV PATH $GOPATH/bin:/usr/local/go/bin:$PATH
ENV XDG_CACHE_HOME /tmp/.cache

RUN mkdir -p "$GOPATH/src" "$GOPATH/bin" && chmod -R 777 "$GOPATH"
#WORKDIR $GOPATH

ARG JOB_TOKEN
ARG JOB_USER
ENV CARGO_NET_GIT_FETCH_WITH_CLI true
WORKDIR /usr/src/opa
COPY . .
RUN apt-get dist-upgrade && apt-get update -y
RUN apt-get install -y build-essential cmake libpthread-stubs0-dev zlib1g-dev zlib1g
#RUN git config --global url."https://${JOB_USER}:${JOB_TOKEN}@gitlab.w6d.io/".insteadOf "https://gitlab.w6d.io/"
RUN ./do_config.sh
RUN rustup component add rustfmt
RUN cargo install --path ./ --locked

FROM debian:bullseye
WORKDIR /usr/local/bin/
RUN apt-get update -y --fix-missing
RUN apt-get install -y build-essential libpq-dev openssl libssl-dev ca-certificates libcurl4
#COPY --from=build /usr/src/opa/configs /usr/local/bin/configs
#COPY --from=build /usr/src/opa/examples /usr/local/bin/examples
COPY --from=build /usr/local/cargo/bin/opa /usr/local/bin/opa
CMD ["opa"]

