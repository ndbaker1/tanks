#!/bin/sh
dist_dir='dist'
package='tanks-worker'

wasm-pack build $package --target web --out-dir ../$dist_dir
cp $package/index.html $dist_dir/index.html