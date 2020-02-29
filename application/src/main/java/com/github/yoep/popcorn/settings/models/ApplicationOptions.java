package com.github.yoep.popcorn.settings.models;

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
    private final boolean bigPictureModeActivated;
    /**
     * Indicates if the kiosk mode is enabled.
     */
    private final boolean kioskModeActivated;
}
