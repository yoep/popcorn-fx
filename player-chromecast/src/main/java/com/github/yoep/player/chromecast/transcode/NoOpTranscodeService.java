package com.github.yoep.player.chromecast.transcode;

import com.github.yoep.player.chromecast.services.TranscodeService;
import com.github.yoep.player.chromecast.services.TranscodeState;
import lombok.extern.slf4j.Slf4j;

@Slf4j
public record NoOpTranscodeService() implements TranscodeService {
    @Override
    public TranscodeState getState() {
        return TranscodeState.STOPPED;
    }

    @Override
    public String transcode(String url) {
        log.warn("Unable to transcode url, NO-OP transcode service is being used");
        return url;
    }

    @Override
    public void stop() {
        // no-op
    }
}