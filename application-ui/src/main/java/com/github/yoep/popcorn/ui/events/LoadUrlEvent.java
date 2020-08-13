package com.github.yoep.popcorn.ui.events;

import lombok.EqualsAndHashCode;
import lombok.Getter;
import org.springframework.util.Assert;

@Getter
@EqualsAndHashCode(callSuper = false)
public class LoadUrlEvent extends LoadEvent {
    /**
     * The url that needs to be loaded.
     */
    private final String url;

    public LoadUrlEvent(Object source, String url) {
        super(source);
        Assert.notNull(url, "url cannot be null");
        this.url = url;
    }
}
