#!/bin/bash
# TODO: Replace this with some proper tool
# TODO: Detect if the package has changed and a release is required

set -e

cd crates/core/
cargo publish
cd ../../

cd crates/procedure/
cargo publish
cd ../../

cd crates/legacy/
cargo publish
cd ../../

cd integrations/axum/
cargo publish
cd ..

cd rspc/
cargo publish
cd ..
