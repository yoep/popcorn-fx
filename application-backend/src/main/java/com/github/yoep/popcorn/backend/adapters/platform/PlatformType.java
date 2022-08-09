package com.github.yoep.popcorn.backend.adapters.platform;

import lombok.Getter;

@Getter
public enum PlatformType {
    WINDOWS("windows"),
    MAC("macos"),
    DEBIAN("debian");

    private final String name;

    PlatformType(String name) {
        this.name = name;
    }
}