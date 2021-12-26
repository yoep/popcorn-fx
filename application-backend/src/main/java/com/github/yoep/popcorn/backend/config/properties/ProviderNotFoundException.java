package com.github.yoep.popcorn.backend.config.properties;

import lombok.Getter;

@Getter
public class ProviderNotFoundException extends RuntimeException {
    private final String name;

    public ProviderNotFoundException(String name) {
        super("Provider '" + name + "' couldn't be found");
        this.name = name;
    }
}
