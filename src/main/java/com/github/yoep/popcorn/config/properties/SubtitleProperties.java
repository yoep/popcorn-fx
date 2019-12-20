package com.github.yoep.popcorn.config.properties;

import lombok.Data;

import javax.validation.constraints.NotEmpty;
import javax.validation.constraints.NotNull;
import java.net.URI;

@Data
public class SubtitleProperties {
    /**
     * The base url of the subtitle provider.
     */
    @NotNull
    private URI url;
    /**
     * The user agent under which is communicated with the subtitle provider.
     */
    @NotEmpty
    private String userAgent;
}
