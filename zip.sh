#!/bin/sh

zip -r "solution_$(date +"%Y-%m-%d_%H-%M-%S").zip" src Cargo.toml
