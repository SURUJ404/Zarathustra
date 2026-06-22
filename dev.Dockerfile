FROM rust:latest

RUN useradd -u 1000 -m zarathustra

COPY ./scripts/install_foundry.sh /tmp/
RUN /tmp/install_foundry.sh

USER zarathustra

WORKDIR /home/zarathustra

COPY --chown=zarathustra:zarathustra . Zarathustra
