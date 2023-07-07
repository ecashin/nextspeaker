#! /bin/sh

out_file="${1?}"
exec > "$out_file"
n_reps=100
set -xe
echo half_life,selection
while read half_life; do
  for i in `seq $n_reps`; do
    printf "%s," "$half_life"
    cargo run -- \
      --history-halflife "$half_life" \
      --history history-abc.txt \
      participants-abc.txt
    done
done <<EOF
0.1
1
10
100
1000
EOF
