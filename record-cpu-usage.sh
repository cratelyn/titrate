#!/usr/bin/env bash
#
# records cpu usage by server pods while the cluster is running.

set -euo pipefail

# record 2 minutes of observations, one snapshot per second.
total=120
interval=1
outfile="cpu-report.csv"

# do not overwrite any preëxisting files.
if [[ -f $outfile ]];
then
  echo "error: $outfile already exists"
  exit 1
fi

# write a header to the file
echo "timestamp,snapshot_id,pod,container,cpu,mem" > $outfile

# record snapshots at a regular interval.
for i in $(seq $total);
do
  echo "snapshot #$i"
  # get the cpu and memory usage for each container in the `titrate` namespace,
  # writing a new row to the output file.
  kubectl --namespace titrate top pod \
    --no-headers --containers \
    | tr -s ' ' | sed -e 's/ $//g' | sed -e 's/ /,/g' \
    | sed -e "s/^/$i,/" | sed -e "s/^/\"$(date '+%Y-%m-%d %H:%M:%S' --utc)\",/" \
    | tee --append $outfile
  sleep $interval
done
