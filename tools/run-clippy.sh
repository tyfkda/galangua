#!/bin/sh

set -e

cargo clippy -- -A clippy::erasing_op -A clippy::identity_op
