#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-or-later
set -eu
: "${SDK_IMAGE:?}" "${TARGET:?}"
VALIDATOR_COMMIT=7dd7dd5

if [ "${1:-}" != --in-sdk ]; then
    root=$(cd "$(dirname "$0")/../.." && pwd)
    work=$(mktemp -d)
    trap 'rm -rf "$work" 2>/dev/null || sudo rm -rf "$work"' EXIT
    git clone -q https://github.com/sailfishos/sdk-harbour-rpmvalidator.git "$work/.validator"
    git -C "$work/.validator" checkout -q "$VALIDATOR_COMMIT"
    (cd "$root" && cp -r --parents RPMS tests/qml tests/translations tools/ci doc/harbour-warnings.md "$work/")
    chmod -R a+rwX "$work"
    docker run --rm ${DOCKER_ARGS:-} -e TARGET -e SDK_IMAGE -v "$work:/home/mersdk/src" -w /home/mersdk/src \
        "$SDK_IMAGE" bash -euc "${SDK_PREPARE:-true}; sh tools/ci/check-rpm.sh --in-sdk"
    exit 0
fi

status=0
fail() {
    echo "FAIL: $1"
    status=1
}

rpm=$(ls RPMS/harbour-nanunana-*.aarch64.rpm | grep -v -e debuginfo -e debugsource | head -n 1)
echo "== $rpm"
rpm -qp --requires "$rpm" | sed 's/^/  requires: /'

validator_status=0
out=$(cd .validator && ./rpmvalidation.sh -c -g . "../$rpm" 2>&1) || validator_status=$?
echo "$out"
if [ "$validator_status" -ne 0 ] || echo "$out" | grep -q '^ERROR'; then
    fail "rpmvalidator reported errors"
fi
echo "$out" | sed -n 's/^WARNING //p' | while IFS= read -r warning; do
    if ! grep -qF -- "$warning" doc/harbour-warnings.md; then
        echo "FAIL: undocumented validator warning: $warning"
        echo undocumented >> /tmp/nanunana-undocumented
    fi
done
[ ! -f /tmp/nanunana-undocumented ] || fail "undocumented validator warnings (doc/harbour-warnings.md)"

work=$(mktemp -d)
(cd "$work" && rpm2cpio "$OLDPWD/$rpm" | cpio -idm --quiet)
bin=$work/usr/bin/harbour-nanunana
nm -D "$bin" | grep -qE ' T main$' || fail "main is not exported"
nm -D "$bin" | grep -q '__libc_start_main@GLIBC_2.34' || fail "__libc_start_main@GLIBC_2.34 not linked"
if readelf -d "$bin" | grep -E 'RPATH|RUNPATH'; then
    fail "binary carries a library search path"
fi
allowed="libc.so.6 libm.so.6 libpthread.so.0 libdl.so.2 librt.so.1 libgcc_s.so.1 libstdc++.so.6 \
libQt5Core.so.5 libQt5Gui.so.5 libQt5Qml.so.5 libQt5Quick.so.5 libsailfishapp.so.1 ld-linux-aarch64.so.1"
for needed in $(readelf -d "$bin" | sed -n 's/.*Shared library: \[\(.*\)\]/\1/p'); do
    case " $allowed " in
        *" $needed "*) echo "  needs $needed" ;;
        *) fail "links $needed, not in the allowed list" ;;
    esac
done
licence=$work/usr/share/harbour-nanunana/LICENSE
if ! grep -q 'GNU GENERAL PUBLIC LICENSE' "$licence" || ! grep -q 'Version 3, 29 June 2007' "$licence"; then
    fail "GPL 3 text not shipped in /usr/share/harbour-nanunana/LICENSE"
fi
size=$(stat -c %s "$bin")
[ "$size" -le 20971520 ] || fail "binary is $size bytes, over the 20 MB budget"

tls_align=$(readelf -lW "$bin" | awk '$1 == "TLS" { print $NF }')
[ $((tls_align)) -ge 128 ] || fail "TLS segment alignment is ${tls_align:-missing}, needs 128 (libhybris)"

desktop=$work/usr/share/applications/harbour-nanunana.desktop
grep -qx 'X-Nemo-Application-Type=silica-qt5' "$desktop" || fail "desktop file is not silica-qt5"
grep -qx 'Permissions=Internet;Location' "$desktop" || fail "sandbox permissions are not Internet;Location"

sb2 -t "$TARGET" -m sdk-install -R zypper --non-interactive in --allow-unsigned-rpm "$rpm"
if sb2 -t "$TARGET" ldd /usr/bin/harbour-nanunana | grep 'not found'; then
    fail "unresolved libraries"
fi

data="$PWD/tests/qml/data"
qml_files=$(find "$PWD/tests/qml" \( -path '*/canary' -o -path '*/data' \) -prune -o -name 'tst_*.qml' -print | sort)
rm -rf "$PWD/qml"
cp -r "/srv/mer/targets/$TARGET/usr/share/harbour-nanunana/qml" "$PWD/qml"
shipped=$(cd "/srv/mer/targets/$TARGET" && find usr/share/harbour-nanunana/qml -name '*.qml' | sort | sed 's|^|/|')
share="$PWD/.qml-share"
rm -rf "$share"
mkdir -p "$share"
qml_check() {
    h=$1
    shift
    mkdir -p "$h"
    sb2 -t "$TARGET" dbus-run-session -- env HOME="$h" QT_QPA_PLATFORM=minimal \
        /usr/bin/harbour-nanunana --qml-check -I /usr/share/harbour-nanunana/qml \
        -T /usr/share/harbour-nanunana/translations -D "$data" "$@"
}
qml_check "$share/shipped.home" $shipped || fail "shipped QML on the target"
n=0
for file in $qml_files; do
    n=$((n + 1))
    qml_check "$share/$n.home" "$file" || fail "page test $file"
done
[ "$n" -gt 0 ] || fail "no page tests found"
if qml_check "$share/canary.home" "$PWD/tests/qml/canary/tst_must_fail.qml" > "$share/canary.log" 2>&1; then
    cat "$share/canary.log"
    fail "QML check passed the canary test that reports a failure"
fi
crash="$share/crash"
mkdir -p "$crash"
sb2 -t "$TARGET" env HOME="$crash" QT_QPA_PLATFORM=minimal sh -c '
    /usr/bin/harbour-nanunana --qml-check "$1" > "$2.out" 2> "$2.err" &
    pid=$!
    tries=0
    until grep -q "canary waits for its crash" "$2.err" || [ "$tries" -ge 120 ]; do
        tries=$((tries + 1))
        sleep 1
    done
    kill -SEGV "$pid"
    wait "$pid"' sh "$PWD/tests/qml/canary/waits_for_crash.qml" "$crash/log" || true
if ! grep -q '^CRASH   signal 11 in thread ' "$crash/log.err" || ! grep -q 'harbour-nanunana(.*\[0x' "$crash/log.err"; then
    cat "$crash/log.out" "$crash/log.err"
    fail "QML check killed by SIGSEGV printed no backtrace"
else
    sed -n '/^CRASH/,$p' "$crash/log.err"
fi
home="$PWD/.qml-home"
rm -rf "$home"
mkdir -p "$home"
if ! sb2 -t "$TARGET" env HOME="$home" QT_QPA_PLATFORM=minimal \
        /usr/bin/harbour-nanunana --qml-check -T /usr/share/harbour-nanunana/translations \
        "$PWD/tests/translations/tst_english_plurals.qml"; then
    fail "English plural forms on the target"
fi
rm -rf "$work" "$home" "$share"
exit $status
