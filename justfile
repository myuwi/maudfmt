help:
  @just --list

test:
  cargo insta test --review

test-watch:
  cargo watch --ignore "snapshots/*.new" -x test
