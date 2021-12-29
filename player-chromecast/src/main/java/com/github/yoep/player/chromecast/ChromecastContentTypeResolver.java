package com.github.yoep.player.chromecast;

import com.github.yoep.player.chromecast.model.VideoMetadata;

import java.net.URI;

public interface ChromecastContentTypeResolver {
    /**
     * Resolve the media metadata of the given video uri.
     *
     * @param uri The uri to resolve the content type of.
     * @return Returns the resolved metadata of the video uri.
     */
    VideoMetadata resolve(URI uri);
}
