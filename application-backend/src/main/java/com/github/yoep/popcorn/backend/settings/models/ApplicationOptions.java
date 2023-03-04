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
     * Indicates if the big picture mode is enabled.
     */
    private final boolean bigPictureMode;
    /**
     * Indicates if the kiosk mode is enabled.
     */
    private final boolean kioskMode;
    /**
     * Indicates if the mouse should be permanently disabled from the application.
     */
    private final boolean mouseDisabled;
}
