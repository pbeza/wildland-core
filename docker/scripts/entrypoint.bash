#!/usr/bin/env bash

if [ -n "$1" ]; then
    case $1 in
        csharp)
            bash /scripts/bindings_csharp-x64-86-linux.bash
            ;;
        *)
            bash
            ;;
    esac
else
    bash
fi
