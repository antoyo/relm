#!/bin/bash

git pull
git push

cd ./relm-derive
cargo release publish --execute
cd ..

cargo release publish --execute
git push

cd ./relm-test
cargo release publish --execute
cd ..
