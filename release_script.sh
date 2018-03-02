#!/bin/bash

cd ./relm-core
cargo release --no-dev-version
cd ..

cd ./relm-state
cargo release --no-dev-version
cd ..

cd ./relm-gen-widget
cargo release --no-dev-version
cd ..

cd ./relm-attributes
cargo release --no-dev-version
cd ..

cd ./relm-derive-common
cargo release --no-dev-version
cd ..

cd ./relm-derive
cargo release --no-dev-version
cd ..

cd ./relm-test
cargo release --no-dev-version
cd ..

cd ./relm-derive-state
cargo release --no-dev-version
cd ..

cp relm-state/src/macros.rs src/macros.rs
git commit . -m "Temporary fix to publish the crate"
cargo release --no-dev-version
git revert HEAD
