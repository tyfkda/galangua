#!/bin/sh

set -e

cargo clippy -- \
    -A clippy::erasing_op \
    -A clippy::identity_op \
    -A clippy::too_many_arguments \
    -A clippy::module_inception \
    -A clippy::missing_safety_doc
