package com.github.yoep.torrent.stream.services;

import lombok.extern.slf4j.Slf4j;
import org.springframework.core.io.Resource;
import org.springframework.http.MediaType;
import org.springframework.http.MediaTypeFactory;
import org.springframework.stereotype.Service;
import org.springframework.util.Assert;

@Slf4j
@Service
public class MediaService {
    /**
     * Retrieve the content type for the given video resource.
     * If the media type couldn't be determined, {@link MediaType#APPLICATION_OCTET_STREAM} will be returned.
     *
     * @param video The resource to retrieve the content type of.
     * @return Returns the content type of the video.
     */
    public MediaType contentType(Resource video) {
        Assert.notNull(video, "video cannot be null");
        var mediaType = MediaTypeFactory.getMediaType(video)
                .orElse(MediaType.APPLICATION_OCTET_STREAM);

        log.trace("Resolved video file \"{}\" as content type \"{}\"", video.getFilename(), mediaType);
        return mediaType;
    }
}
