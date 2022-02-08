#!/bin/bash

git pull
git push

cd ./relm-derive
cargo release --no-dev-version --execute
cd ..

cargo release --no-dev-version --execute
git push

cd ./relm-test
cargo release --no-dev-version --execute
cd ..
