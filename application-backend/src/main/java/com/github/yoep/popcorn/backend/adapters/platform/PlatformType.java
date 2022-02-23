package com.github.yoep.popcorn.backend.adapters.platform;

import lombok.Getter;

@Getter
public enum PlatformType {
    DEBIAN("debian"),
    MAC("macos"),
    WINDOWS("windows");

    private final String name;

    PlatformType(String name) {
        this.name = name;
    }
}
