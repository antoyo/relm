#!/bin/bash

git pull
git push

cd ./relm-derive
cargo release --execute
cd ..

cargo release --execute
git push

cd ./relm-test
cargo release --execute
cd ..
