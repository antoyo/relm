#!/bin/bash

rustup default nightly

git pull
git push

cd ./relm-derive
cargo release --no-dev-version
cd ..

cd ./relm-test
cargo release --no-dev-version
cd ..

cargo release --no-dev-version
git push
