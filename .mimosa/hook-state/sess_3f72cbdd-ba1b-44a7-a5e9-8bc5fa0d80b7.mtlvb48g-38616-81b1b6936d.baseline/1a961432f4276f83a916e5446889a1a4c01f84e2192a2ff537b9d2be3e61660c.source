#!/bin/bash
for file in $(find tooling/packages/ -name "package.json"); do
    sed -i 's/"version": "0.13.1"/"version": "0.14.0"/g' $file
    sed -i 's/"version": "0.1.0"/"version": "0.14.0"/g' $file
done
