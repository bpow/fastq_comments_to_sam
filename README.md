# Add comments from fastq files to corresponding SAM files

## TL;DR

```bash
samtools view --no-PG -h input.cram | fastq_comments_to_sam \
   <(xzcat read1.fastq.xz | awk 'NR % 4 == 1') \
   <(xzcat read2.fastq.xz | awk 'NR % 4 == 1') \
   | samtools --no-PG -T ref.fa -O cram,small --write-index -o output.cram
```

## Why would you want this?

Say you have a bunch of CRAM or BAM files that you've processed from your initial FASTQs, but you
didn't happen to include the read comments in your CRAM files that you want for archival purposes.
Maybe you're not sure if you'll ever need those FASTQ header comments, but you don't want to
chance it...

## What it does

Reads FASTQ header lines from command line argument(s) to parse out the read names and header
comments into an in-memory hash (with some hacky hash tables to try to be as memory efficient
as possuble). If these are formatted like CASAVA comments, then the barcode information is what
is stored. If it already looks like a SAM tag, the whole comment is stored, otherwise it stores
the comment with an `XC:Z:` prefix so it can be incorporated into the SAM output.

Then it reads SAM input (from stdin) and looks up the read name, adding the corresponding
SAM tag representing the FASTQ header comment to the output.
