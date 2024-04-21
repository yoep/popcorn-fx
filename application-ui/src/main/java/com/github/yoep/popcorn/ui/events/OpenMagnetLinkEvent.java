package com.github.yoep.popcorn.ui.events;

import com.github.yoep.popcorn.backend.events.ApplicationEvent;
import lombok.EqualsAndHashCode;
import lombok.Getter;

import java.util.Objects;

@Getter
@EqualsAndHashCode(callSuper = false)
public class OpenMagnetLinkEvent extends ApplicationEvent {
    /**
     * The magnet url link to open.
     */
    private final String url;

    public OpenMagnetLinkEvent(Object source, String url) {
        super(source);
        Objects.requireNonNull(url, "url cannot be null");
        this.url = url;
    }
}
