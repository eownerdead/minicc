#!/usr/bin/env sh

DIR=$(cd "$(dirname "$0")" && pwd)

CC=${CC:-"/usr/bin/cc"}

CCOM="$DIR/../target/debug/ccom"

compile() {
    file="$1"

    $CCOM > /tmp/minicc_test.s < "$DIR/$file" &&
    $CC -m32 -o /tmp/minicc_test /tmp/minicc_test.s "$DIR/../lib/dbg.c"
}

test() {
    name="$1"

    printf "%s " "$name"

    compile "$name.c"

    if /tmp/minicc_test 2>&1 | diff -u "$DIR/$name.expect" -; then
        echo "=> OK"
    else
        echo "=> FAILED"
    fi
}

for i in "$DIR"/*.c; do
    test "$(basename "$i" .c)"
done
