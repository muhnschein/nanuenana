#!/bin/bash
# SPDX-License-Identifier: GPL-3.0-or-later
set -euo pipefail
mb2 -t "$1" build -j "$(nproc)"
