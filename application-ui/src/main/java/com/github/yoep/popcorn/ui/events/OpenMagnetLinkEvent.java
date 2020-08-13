package com.github.yoep.popcorn.ui.events;

import lombok.EqualsAndHashCode;
import lombok.Getter;
import org.springframework.context.ApplicationEvent;
import org.springframework.util.Assert;

@Getter
@EqualsAndHashCode(callSuper = false)
public class OpenMagnetLinkEvent extends ApplicationEvent {
    /**
     * The magnet url link to open.
     */
    private final String url;

    public OpenMagnetLinkEvent(Object source, String url) {
        super(source);
        Assert.notNull(url, "url cannot be null");
        this.url = url;
    }
}
