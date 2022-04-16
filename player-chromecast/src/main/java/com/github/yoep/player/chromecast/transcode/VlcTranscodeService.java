package com.github.yoep.player.chromecast.transcode;

import com.github.yoep.player.chromecast.services.TranscodeService;
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

import static uk.co.caprica.vlcj.binding.LibVlc.libvlc_errmsg;

@Slf4j
@ToString
@EqualsAndHashCode
@RequiredArgsConstructor
public class VlcTranscodeService implements TranscodeService {
    static final String EXTENSION = "mp4";

    private final MediaPlayerFactory mediaPlayerFactory;
    private final MediaPlayerEventListener listener = createListener();

    private MediaPlayer mediaPlayer;

    @Override
    public String transcode(String url) {
        Objects.requireNonNull(url, "url cannot be null");
        log.trace("Starting transcoding of {}", url);
        var baseName = FilenameUtils.getBaseName(url);
        var name = baseName + "." + EXTENSION;
        var destination = MessageFormat.format("{0}:{1}/{2}", HostUtils.hostAddress(), String.valueOf(HostUtils.availablePort()), name);

        // release the previous resources if needed
        releaseMediaPlayer();

        // create a new media player
        mediaPlayer = mediaPlayerFactory.mediaPlayers().newMediaPlayer();
        mediaPlayer.events().addMediaPlayerEventListener(listener);

        var started = mediaPlayer.media().play(url, ":sout=#transcode{vcodec=h264,vb=2048,acodec=mp3,ab=128,channels=2,threads=0,deinterlace}:" +
                "http{mux=ffmpeg{mux=mp4},dst=" + destination + "}", ":sout-keep");

        if (!started) {
            throw new TranscodeException("Failed to start transcoding of " + url);
        }

        log.info("Converted video is available at http://{}", destination);
        return "http://" + destination;
    }

    @Override
    public void stop() {
        releaseMediaPlayer();
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
            }

            @Override
            public void buffering(MediaPlayer mediaPlayer, float newCache) {
                log.trace("Transcoding is currently buffering at {}", newCache);
            }

            @Override
            public void error(MediaPlayer mediaPlayer) {
                var message = libvlc_errmsg();
                log.error("Transcoding has failed, {}", message);
            }
        };
    }

    private void releaseMediaPlayer() {
        Optional.ofNullable(mediaPlayer).ifPresent(e -> {
            try {
                log.debug("Releasing the transcode process");
                e.controls().stop();
                e.release();
            } catch (Throwable ex) {
                log.error("Failed to release transcode process, {}", ex.getMessage(), ex);
            }
        });
    }
}
