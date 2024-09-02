#!/bin/bash
set -euo pipefail
IFS=$'\n\t'

### WARNING: This is pretty specific to my setup. You'll need to modify it to suit your needs.
### Best of luck!

CRAMIN=$1
shift
FASTQ=$@
CRAMOUT=$(basename $CRAMIN .cram).archive.cram
sbatch -t 12:00:00 --mem=32G -c 6 <<EOF
#!/usr/bin/bash

set -xeuo pipefail
IFS=\$'\n\t'

module load samtools/1.20
module load human-genome-for-alignment/1405.15

samtools view -h --no-PG -@ 2 "$CRAMIN" \
  | /usr/bin/time -v fastq_comments_to_sam \
      <(xzcat $FASTQ | awk 'NR%4 == 1') \
  | samtools view -h --write-index --no-PG -T "\$GENOME_FOR_ALIGNMENT" -Ocram,archive,level=8,embed_ref=1 -o "$CRAMOUT" -@ 4
EOF
