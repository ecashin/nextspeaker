#! /bin/sh
set -xe
for i in index-*.css nextspeaker-web-*_bg.wasm nextspeaker-web-*.js; do
    if test -r "$i"; then
        git rm -f "$i"
    fi
done
ls nextspeaker-web/dist/index-*.css \
    nextspeaker-web/dist/nextspeaker-web-*_bg.wasm \
    nextspeaker-web/dist/nextspeaker-web-*.js | while read i; do
    b=`basename "$i"`
    cp "$i" "$b"
    git add "$b"
done
cp nextspeaker-web/dist/index.html .
