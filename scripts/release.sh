#!/usr/bin/fish
# TODO:
#   Make available to run using bash or sh
#   Read version number from cargo file
#   There's got to be a better way to do this
set -x CROSS_DOCKER_IN_DOCKER true
set -x TARGET armv7-unknown-linux-gnueabihf

cross build --release --examples --target $TARGET
cross build --release --target $TARGET

set -x EXAMPLES (find ./examples -type f -iname "*.rs" -exec basename {} .rs ';')

if test -d ./target/$TARGET/package
    rm -rf ./target/$TARGET/package
end

if test -e ./target/caminatus-0.0.0.tar.gz
    rm ./target/caminatus-0.0.0.tar.gz
end

mkdir -p ./target/$TARGET/package/0.0.0/examples

cp ./target/$TARGET/release/caminatus ./target/$TARGET/package/0.0.0
cp ./config.yaml.example ./target/$TARGET/package

for EXAMPLE in $EXAMPLES
    cp ./target/$TARGET/release/examples/$EXAMPLE ./target/$TARGET/package/0.0.0/examples
end

tar -czvf ./target/caminatus-0.0.0.tar.gz ./target/$TARGET/package