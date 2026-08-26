# tapline in a container, with nothing else in it.
#
# No steamcmd, no Steam client, no 32-bit runtime, no CA bundle. tapline speaks
# Steam's CM protocol itself and `rustls` carries Mozilla's roots compiled in,
# so the final image is one static binary on `scratch` and nothing else.
#
#   docker build -t tapline .
#   docker run --rm -v /srv/gmod:/data tapline \
#     +login anonymous +force_install_dir /data +app_update 4020 +quit

FROM rust:alpine AS build

# musl-dev for the C that `ring` compiles; there is no way around that one and
# the README says so rather than claiming a pure-Rust build.
RUN apk add --no-cache musl-dev

WORKDIR /src
COPY . .

RUN cargo build --release --target x86_64-unknown-linux-musl -p tapline-cli \
 && strip target/x86_64-unknown-linux-musl/release/tapline

# `scratch`, not alpine. There is nothing for a shell to do here, and an image
# with no shell is an image with no shell to be dropped into.
FROM scratch

COPY --from=build /src/target/x86_64-unknown-linux-musl/release/tapline /tapline

# A writable place for downloads; mount a volume over it.
WORKDIR /data

ENTRYPOINT ["/tapline"]
