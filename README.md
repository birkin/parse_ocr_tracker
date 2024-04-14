## Description...

This app parses the ocr-tracker files and creates a CSV file for analysis in OpenRefine or a spreadsheet.

Background...

For the Hall-Hoag project, the expected 800,000+ items are far too many to carefully spot-check and address the inevitable OCR issues that will be discovered.

The question: Down the road, when time and labor are available, how might we most efficiently identify the items that should be first checked for possible problems?

For the ingestion-process, we created tracker files for the purpose of efficiently determining which items had already been OCRed and ingested. We then decided, during the OCR process, to save some of the OCR "confidence" stats in the OCR-tracker files -- the thought being we could later process these files to find the ingested BDR-items most likely to need checking. Thus this ocr-tracker parser-app.

Note that this app is written in Rust. Our dev-team doesn't code in Rust; we're a Python shop. This was coded outside of work-hours as a side-project to deepen one of the dev's knowledge of Rust. It's a great project for Rust: its speed and memory-efficient concurrency enable quick processing of a _very_ large amount of data.

---


## Usage...

for development:

`% cargo run -- --source_dir_path "foo" --output_dir_path "bar"`

for binary:

`% parse_ocr_tracker --help`

`% parse_ocr_tracker --source_dir_path "foo" --output_dir_path "bar"`

The returned json shows the path to the csv file, as well as other useful info.

---
