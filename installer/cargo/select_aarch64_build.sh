#!/usr/bin/env bash

cargo_src=`cat Cargo.SOURCE`

if [ "$cargo_src" = 'aarch64' ]; then
	echo 'AARCH64 build already selected; exiting now...'
	exit 0
fi
echo 'Switching build process over to AARCH64 build...'

cp cargo/aarch64/* .