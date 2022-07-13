#!/usr/bin/env sh

DIR=$(cd "$(dirname "$0")" && pwd)

CC=${CC:-"/usr/bin/cc"}

CCOM="$DIR/../target/debug/ccom"

compile() {
    program="$1"

    echo "$program" | $CCOM > /tmp/minicc_test.s &
    $CC -m32 -o /tmp/minicc_test /tmp/minicc_test.s
}

test_case() {
    program="$1"
    expect="$2"

    printf "%s " "$program"

    compile "$program"

    /tmp/minicc_test
    result="$?"
    if [ "$result" = "$expect" ]; then
        echo "=> OK"
    else
        echo "=> FAILED. expect $expect, result $result"
    fi
}

test_case "{}" 0
test_case "{return 42; }	" 42
test_case "{int a; a=12+ 13+ 14;}" 0
test_case "   { return	64  -4 ;} " 60
test_case " {return 4*	( 3+2)% 7; } " 6
test_case " {	321 /43+ 12 ; 3	/2-1   +123;return 9 /4*(3 +2);} " 10
test_case "{ int compiler; int b; compiler = 32 + 4; b = compiler - 15; return b; }" 21
test_case "{ int foo; int bar; foo = 3; bar = 5; return foo == bar; }" 0
test_case "{ return 42 > 42; }" 0
test_case "{ return 42 >= 42; }" 1
test_case "{ return !0 - !1; }" 1
test_case "{ if (1 + 1 == 3) { return 5; } else { return 9; } }" 9
test_case "{ int a; a = 54; if (6 * 9 == a) { return 8; } return a - 12; }" 8
test_case "{ if (99 > 111) { return 100; } return 4; }" 4
test_case "{ if (32 != 31) { 1 + 1; } return 10; }" 10
