#!/bin/bash

# Exit if any subcommand fails
set -e

# Get tag
TAG=$(cat ./zarathustra_cli/Cargo.toml | grep '^version' | awk '{print $3}' | sed -e 's/"//g') && echo $TAG

# Use zarathustra github bot
git config --global user.email $GH_USER

# Release on Dockerhub

## Build
docker build -t zarathustra .

## Log into Dockerhub
echo $DOCKERHUB_PASS | docker login -u $DOCKERHUB_USER --password-stdin

## Release under `latest` tag
docker tag zarathustra:latest zarathustra/zarathustra:latest
docker push zarathustra/zarathustra:latest
echo "Published zarathustra/zarathustra:latest"

## Release under $TAG tag
docker tag zarathustra:latest zarathustra/zarathustra:$TAG
docker push zarathustra/zarathustra:$TAG
echo "Published zarathustra/zarathustra:$TAG"

# Release on Github
git tag -f latest
git tag -f $TAG
git push origin -f latest
git push origin $TAG

# Create a release draft
curl \
  -X POST \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: token $GH_TOKEN" \
  https://api.github.com/repos/Zarathustra/Zarathustra/releases \
  -d "{\"tag_name\":\"$TAG\",\"draft\":true}"

# Build zarathustra js
docker build -t zarathustra_js -f zarathustra_js/Dockerfile .

CID=$(docker create zarathustra_js)

docker cp ${CID}:/build zarathustra_js/dist
docker rm -f ${CID}

cd zarathustra_js/dist

# Publish zarathustra_js to npmjs
chmod +x publish.sh
./publish.sh

# Publish book
MDBOOK_TAR="https://github.com/rust-lang-nursery/mdBook/releases/download/v0.2.1/mdbook-v0.2.1-x86_64-unknown-linux-gnu.tar.gz"

cd ../../zarathustra_book

## Install mdbook
wget -qO- $MDBOOK_TAR | tar xvz

## Build book
./mdbook build

## Deploy to github.io
pip3 install ghp-import
git clone https://github.com/Zarathustra/zarathustra.github.io.git && cd zarathustra.github.io
ghp-import -n -p -f -m "Documentation upload. Version:  $TAG" -b "master" -r https://zarathustrabot:"$GH_TOKEN"@github.com/Zarathustra/zarathustra.github.io.git ../book
echo "Published book"