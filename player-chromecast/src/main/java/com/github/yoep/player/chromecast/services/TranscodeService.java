package com.github.yoep.player.chromecast.services;

import javax.validation.constraints.NotNull;

public interface TranscodeService {
    /**
     * Get the state of the transcoding process.
     *
     * @return Returns the transcoding state.
     */
    TranscodeState getState();

    /**
     * Transcode the given original video url.
     *
     * @param url The original url to transcode.
     * @return Returns the new url which contains the converted video.
     */
    @NotNull
    String transcode(@NotNull String url);

    /**
     * Stop the transcoding process.
     */
    void stop();
}
