package com.github.yoep.popcorn.backend.settings.models;

import lombok.Builder;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

@Getter
@ToString
@EqualsAndHashCode
@Builder
public class ApplicationOptions {
    /**
     * Indicates if the mouse should be permanently disabled from the application.
     */
    private final boolean mouseDisabled;
}
