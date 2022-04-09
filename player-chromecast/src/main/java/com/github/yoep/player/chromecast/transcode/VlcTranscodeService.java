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
    static final String EXTENSION = "webm";

    private final MediaPlayerFactory mediaPlayerFactory;
    private final MediaPlayerEventListener listener = createListener();

    private MediaPlayer mediaPlayer;

    @Override
    public String transcode(String url) {
        Objects.requireNonNull(url, "url cannot be null");
        log.trace("Starting transcoding of {}", url);
        var baseName = FilenameUtils.getBaseName(url);
        var name = baseName + "." + EXTENSION;
        var port = HostUtils.availablePort();

        // release the previous resources if needed
        releaseMediaPlayer();

        // create a new media player
        mediaPlayer = mediaPlayerFactory.mediaPlayers().newMediaPlayer();
        mediaPlayer.events().addMediaPlayerEventListener(listener);

        var started = mediaPlayer.media().play(url, ":sout=#transcode{vcodec=VP80,vb=1000,vfilter=canvas{width=640,height=360},acodec=vorb,ab=128,channels=2," +
                "samplerate=44100,threads=2}:http{mux=" + EXTENSION + ",dst=:" + port + "/" + name + "}", ":sout-keep");

        if (!started) {
            throw new TranscodeException("Failed to start transcoding of " + url);
        }

        return MessageFormat.format("http://{0}:{1}/{2}", HostUtils.hostAddress(), String.valueOf(port), name);
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
                log.debug("Opening");
            }

            @Override
            public void buffering(MediaPlayer mediaPlayer, float newCache) {
                log.debug("Buffering");
            }

            @Override
            public void error(MediaPlayer mediaPlayer) {
                var message = libvlc_errmsg();
                log.error("Transcoding failed, {}", message);
            }
        };
    }

    private void releaseMediaPlayer() {
        Optional.ofNullable(mediaPlayer).ifPresent(MediaPlayer::release);
    }
}
