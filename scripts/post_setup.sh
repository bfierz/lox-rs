#!/bin/bash

sudo apt-get update
sudo apt-get install -y apt-transport-https
sudo sh -c 'wget -qO- https://dl-ssl.google.com/linux/linux_signing_key.pub | apt-key add -'
sudo sh -c 'wget -qO- https://storage.googleapis.com/download.dartlang.org/linux/debian/dart_stable.list > /etc/apt/sources.list.d/dart_stable.list'
sudo apt-get update
sudo apt-get install -y dart
export PATH="$PATH:/usr/lib/dart/bin"
source ~/.bashrc

dart --version
rustc --version
