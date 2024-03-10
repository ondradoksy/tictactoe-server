#!/bin/bash
RUST_BACKTRACE=1 cargo watch -i .gitignore -i target -x check -x test -x run