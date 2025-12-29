#!/bin/sh
# process video for frames.rs
set -euo pipefail

if [ -d "frame" ]; then
    rm -r frame
fi

mkdir -p frame
ffmpeg -i $1 -vf scale=178:128,setsar=1 frame/bin-%d.pbm
for fn in $(ls frame); do
    pnmtopnm -plain frame/$fn > frame/$(echo $fn | cut -c 5-)
    rm frame/$fn
done

python -c 'print("P1\n178 128\n" + "0" * (178 * 128))' > frame/0.pbm
for i in $(seq 2 $(ls frame | wc -l)); do
    cargo run --example frames -- frame/$(echo "$i 2 - p" | dc).pbm frame/$(echo "$i 1 - p" | dc).pbm
done
