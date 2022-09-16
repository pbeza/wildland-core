#!/usr/bin/env bash

if [ -n "$1" ]; then
    case $1 in
        swift)
            bash /scripts/bindings_swift-x64-86-linux.bash
            ;;
        cpp)
            bash /scripts/bindings_cpp-x64-86-linux.bash
            ;;
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
