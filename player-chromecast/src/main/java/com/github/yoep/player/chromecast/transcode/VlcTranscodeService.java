package com.github.yoep.player.chromecast.transcode;

import com.github.yoep.player.chromecast.services.TranscodeService;
import com.github.yoep.player.chromecast.services.TranscodeState;
import com.github.yoep.popcorn.backend.utils.HostUtils;
import lombok.EqualsAndHashCode;
import lombok.RequiredArgsConstructor;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FilenameUtils;
import uk.co.caprica.vlcj.factory.MediaPlayerFactory;
import uk.co.caprica.vlcj.player.base.MediaPlayer;
import uk.co.caprica.vlcj.player.base.MediaPlayerEventAdapter;
import uk.co.caprica.vlcj.player.base.MediaPlayerEventListener;

import javax.annotation.PreDestroy;
import java.text.MessageFormat;
import java.util.Objects;
import java.util.Optional;

@Slf4j
@ToString
@EqualsAndHashCode
@RequiredArgsConstructor
public class VlcTranscodeService implements TranscodeService {
    private final MediaPlayerFactory mediaPlayerFactory;
    private final MediaPlayerEventListener listener = createListener();

    private TranscodeState state = TranscodeState.UNKNOWN;
    private MediaPlayer mediaPlayer;

    @Override
    public TranscodeState getState() {
        return state;
    }

    @Override
    public String transcode(String url) {
        Objects.requireNonNull(url, "url cannot be null");
        log.trace("Starting transcoding of {}", url);
        state = TranscodeState.PREPARING;
        var baseName = FilenameUtils.getBaseName(url);
        var destination = MessageFormat.format("{0}:{1}/{2}", HostUtils.hostAddress(), String.valueOf(HostUtils.availablePort()), baseName);

        // release the previous resources if needed
        releaseMediaPlayer();

        // create a new media player
        mediaPlayer = mediaPlayerFactory.mediaPlayers().newMediaPlayer();
        mediaPlayer.events().addMediaPlayerEventListener(listener);

        var started = mediaPlayer.media().play(url, ":sout=#transcode{vcodec=h264,vb=2048,fps=24,maxwidth=1920,maxheight=1080,acodec=mp3,ab=128,channels=2," +
                "threads=0}:" +
                "std{mux=avformat{mux=matroska,options={live=1},reset-ts},dst=" + destination + ",access=http}", ":demux-filter=demux_chromecast", ":sout-mux" +
                "-caching=8192", ":sout-all", ":sout-keep");

        if (!started) {
            throw new TranscodeException("Failed to start transcoding of " + url);
        }

        log.info("Converted video is available at http://{}", destination);
        return "http://" + destination;
    }

    @Override
    public void stop() {
        releaseMediaPlayer();
        state = TranscodeState.STOPPED;
    }

    @PreDestroy
    void onDestroy() {
        // release the VLC resources
        mediaPlayerFactory.release();
        releaseMediaPlayer();
    }

    private MediaPlayerEventListener createListener() {
        return new MediaPlayerEventAdapter() {
            @Override
            public void opening(MediaPlayer mediaPlayer) {
                log.trace("Transcoding is opening the original video");
                state = TranscodeState.STARTING;
            }

            @Override
            public void buffering(MediaPlayer mediaPlayer, float newCache) {
                log.trace("Transcoding is currently buffering at {}", newCache);
            }

            @Override
            public void playing(MediaPlayer mediaPlayer) {
                log.info("Transcoding of the video has started");
                state = TranscodeState.TRANSCODING;
            }

            @Override
            public void timeChanged(MediaPlayer mediaPlayer, long newTime) {
                log.trace("Transcoding progress at {}", newTime);
            }

            @Override
            public void error(MediaPlayer mediaPlayer) {
                log.error("Failed to transcode video");
                state = TranscodeState.ERROR;
            }
        };
    }

    private void releaseMediaPlayer() {
        Optional.ofNullable(mediaPlayer).ifPresent(e -> {
            try {
                log.debug("Releasing the transcode process");
                e.controls().stop();
                e.release();
                mediaPlayer = null;
            } catch (Throwable ex) {
                log.error("Failed to release transcode process, {}", ex.getMessage(), ex);
            }
        });
    }
}
