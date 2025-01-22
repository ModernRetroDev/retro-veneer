#!/usr/bin/env bash

cargo_src=`cat Cargo.SOURCE`

if [ "$cargo_src" = 'unstable' ]; then
	echo 'Unstable build already selected; exiting now...'
	exit 0
fi
echo 'Switching build process over to unstable build...'

cp cargo/unstable/* .