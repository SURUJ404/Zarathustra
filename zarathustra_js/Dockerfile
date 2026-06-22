FROM zarathustra/env:latest

COPY . src
RUN cd src/zarathustra_js && npm run build && rm -rf target/

RUN cd / && mkdir build \
    && cp -rf src/zarathustra_js/* /build/ \
    && rm -rf src