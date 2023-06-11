#!/bin/bash

set -euxo pipefail

: 'Checking for fpm Ruby gem, installing if not present'
installed=`gem list -i fpm` || true

if [ $installed = 'false' ]; then
  gem install fpm
fi

: 'Running the build'
cargo build

: "Packaging"
fpm \
  -s dir -t deb \
  -p framework-inputmodule-rs-0.0.1.deb \
  --name framework-inputmodule-rs \
  --version 0.0.1 \
  --architecture all \
  --description "framework-inputmodule-rs runs Framework Laptop 16 input modules and keeps their firmware up to date" \
  --url "https://frame.work" \
  --maintainer "Framework <support@frame.work>" \
  --deb-systemd ./framework-inputmodule-rs.service \
  --deb-systemd-enable \
  --deb-systemd-auto-start \
  --deb-systemd-restart-after-upgrade \
  --after-install postinstall.sh \
  target/x86_64-unknown-linux-gnu/debug/inputmodule-control=/usr/bin/framework-inputmodule-rs

: 'Packaging successful, install with "sudo dpkg -i <pkg-name>.deb"'