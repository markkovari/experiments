# --- Build stage: compile a fully static musl binary (sqlite C compiled in). ---
# Pin a Zig version matching the dev toolchain this project targets.
FROM alpine:3.20 AS build

# This project uses 0.17.0-dev-only stdlib API (std.Io, std.Io.net) and the
# post-@cImport extern FFI, so the container MUST use the SAME dev compiler as
# local dev — a released Zig (0.14/0.15) will not compile it.
#
# NOTE: dev-build tarballs are transient; ziglang.org garbage-collects old ones.
# If this URL 404s, bump ZIG_VERSION to a current dev build and update the local
# toolchain (zvm) + build.zig.zon minimum_zig_version to match.
ARG ZIG_VERSION=0.17.0-dev.298+ad1b746e2
RUN apk add --no-cache curl xz \
 && ARCH="$(uname -m)" \
 && curl -sSL "https://ziglang.org/builds/zig-${ARCH}-linux-${ZIG_VERSION}.tar.xz" -o /tmp/zig.tar.xz \
 && mkdir -p /opt/zig && tar -xf /tmp/zig.tar.xz -C /opt/zig --strip-components=1 \
 && ln -s /opt/zig/zig /usr/local/bin/zig \
 && rm /tmp/zig.tar.xz

WORKDIR /src
COPY . .

# Static musl target -> a self-contained binary with no runtime deps.
RUN zig build -Doptimize=ReleaseSafe \
      -Dtarget=$(uname -m)-linux-musl

# --- Runtime stage: minimal image carrying just the static binary. ---
FROM alpine:3.20 AS runtime

# Non-root user owning the data dir (12-factor: processes run unprivileged).
RUN adduser -D -H app && mkdir -p /data && chown app /data

COPY --from=build /src/zig-out/bin/zig_state_machine /usr/local/bin/todo

USER app
WORKDIR /data

# 12-factor config: all behavior comes from the environment. These are defaults;
# override with `docker run -e ...`.
ENV TODO_BACKEND=sqlite \
    TODO_DB_PATH=/data/todos.db \
    HOST=0.0.0.0 \
    PORT=8080

EXPOSE 8080
# State lives in an attached volume, not the container layer (12-factor VI).
VOLUME ["/data"]

CMD ["todo"]
