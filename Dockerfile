FROM zarathustra/env:20.04 as build

WORKDIR /build

COPY . src
RUN cd src; ./build_release.sh

FROM ubuntu:20.04
ENV ZARATHUSTRA_HOME=/home/zarathustra/.zarathustra

RUN useradd -u 1000 -m zarathustra

USER zarathustra
WORKDIR /home/zarathustra

COPY --from=build --chown=zarathustra:zarathustra /build/src/target/release/zarathustra $ZARATHUSTRA_HOME/bin/
COPY --from=build --chown=zarathustra:zarathustra /build/src/zarathustra_cli/examples $ZARATHUSTRA_HOME/examples
COPY --from=build --chown=zarathustra:zarathustra /build/src/zarathustra_stdlib/stdlib $ZARATHUSTRA_HOME/stdlib

ENV PATH "$ZARATHUSTRA_HOME/bin:$PATH"
ENV ZARATHUSTRA_STDLIB "$ZARATHUSTRA_HOME/stdlib"