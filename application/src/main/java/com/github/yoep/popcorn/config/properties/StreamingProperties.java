package com.github.yoep.popcorn.config.properties;

import lombok.Data;
import org.springframework.context.annotation.Configuration;

@Data
@Configuration
public class StreamingProperties {
    /**
     * The default chunk size (in bytes) to use for streaming when the client doesn't specify an end range.
     * Default is 1MB
     */
    private long chunkSize = 1 * 1024 * 1024;
}
