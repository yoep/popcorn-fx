package com.github.yoep.player.vlc.services;

import com.github.yoep.player.vlc.VlcListener;
import com.github.yoep.player.vlc.VlcPlayerConstants;
import com.github.yoep.player.vlc.model.VlcState;
import com.github.yoep.player.vlc.model.VlcStatus;
import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import com.github.yoep.popcorn.backend.subtitles.Subtitle;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.lang.Nullable;
import org.springframework.stereotype.Service;
import org.springframework.web.reactive.function.client.WebClient;
import org.springframework.web.reactive.function.client.WebClientException;
import org.springframework.web.reactive.function.client.WebClientRequestException;
import org.springframework.web.util.UriComponentsBuilder;

import javax.validation.constraints.NotNull;
import java.io.File;
import java.net.ConnectException;
import java.text.MessageFormat;
import java.time.Duration;
import java.util.Objects;
import java.util.Optional;
import java.util.Timer;
import java.util.TimerTask;

@Slf4j
@Service
@RequiredArgsConstructor
public class VlcPlayerService extends AbstractListenerService<VlcListener> {
    static final String OPTIONS = MessageFormat.format("--http-host={0} --http-port={1} --extraintf=http --http-password={2}",
            VlcPlayerConstants.HOST, VlcPlayerConstants.PORT, VlcPlayerConstants.PASSWORD);
    static final String SUBTITLE_OPTION = "--sub-file=";
    static final String STATUS_PATH = "/requests/status.xml";
    static final String COMMAND_NAME_PARAM = "command";
    static final String COMMAND_VALUE_PARAM = "val";

    private final PlatformProvider platformProvider;
    private final SubtitleService subtitleService;
    private final WebClient vlcWebClient;

    Timer statusTimer;

    /**
     * Pla the given request through an external VLC player.
     *
     * @param request The request to play.
     * @return Returns true if the player started with success, else false.
     */
    public boolean play(PlayRequest request) {
        Objects.requireNonNull(request, "request cannot be null");
        var subtitleOption = subtitleService.getActiveSubtitle()
                .filter(e -> !e.isNone())
                .flatMap(Subtitle::getFile)
                .map(File::getAbsolutePath)
                .map(e -> SUBTITLE_OPTION + e)
                .orElse("");
        var command = MessageFormat.format("vlc {0} {1} {2}", request.getUrl(), OPTIONS, subtitleOption).trim();

        log.debug("Launching VLC process ({})", command);
        if (platformProvider.launch(command)) {
            startStatusListener();
            return true;
        }

        return false;
    }

    /**
     * Stop the VLC playback.
     * This will quit the status monitor of the VLC player and purge it's resources.
     */
    public void stop() {
        Optional.ofNullable(statusTimer)
                .ifPresent(e -> {
                    log.debug("Stopping the VLC status thread");
                    statusTimer.cancel();
                    statusTimer.purge();
                    statusTimer = null;
                });
    }

    /**
     * Execute the given command on the VLC player.
     *
     * @param command The command to execute on the VLC player.
     */
    public void executeCommand(@NotNull String command) {
        executeCommand(command, null);
    }

    /**
     * Execute the given command and command value on the VLC player.
     *
     * @param command The command to execute on the VLC player.
     * @param value   The command value to pass along with the command to the VLC player.
     */
    public void executeCommand(@NotNull String command, @Nullable String value) {
        Objects.requireNonNull(command, "command cannot be null");
        var uri = retrieveBaseUriBuilder()
                .queryParam(COMMAND_NAME_PARAM, command);

        // check if a command value should be passed along
        Optional.ofNullable(value)
                .ifPresent(e -> uri.queryParam(COMMAND_VALUE_PARAM, e));

        try {
            log.trace("Executing VLC command {}", uri);
            vlcWebClient.get()
                    .uri(uri.build().toUri())
                    .retrieve()
                    .toBodilessEntity()
                    .block(Duration.ofMillis(1000));
        } catch (WebClientException ex) {
            log.warn("Failed to execute VLC command {}, {}", command, ex.getMessage(), ex);
        }
    }

    private void startStatusListener() {
        // stop an existing status timer if one is present
        stop();

        statusTimer = new Timer("VlcPlaybackStatus");
        statusTimer.schedule(new VlcStatusThread(), 0, 1000);
    }

    private void onStatusChanged(VlcStatus status) {
        if (status == null) {
            log.warn("Invalid VLC status received, ignoring status update");
            return;
        }

        onTimeChanged(status.getTime());
        onDurationChanged(status.getLength());
        onStateChanged(status.getState());
    }

    private void onTimeChanged(Long time) {
        invokeListeners(e -> e.onTimeChanged(time));
    }

    private void onDurationChanged(Long length) {
        invokeListeners(e -> e.onDurationChanged(length));
    }

    private void onStateChanged(VlcState state) {
        invokeListeners(e -> e.onStateChanged(state));
    }

    private UriComponentsBuilder retrieveBaseUriBuilder() {
        return UriComponentsBuilder.newInstance()
                .scheme("http")
                .host(VlcPlayerConstants.HOST)
                .port(VlcPlayerConstants.PORT)
                .path(STATUS_PATH);
    }

    /**
     * Internal task which retrieves the latest media status of the current playback from
     * the {@link com.github.yoep.player.vlc.VlcPlayer}.
     */
    private class VlcStatusThread extends TimerTask {
        private int totalConnectionsRefused;

        @Override
        public void run() {
            try {
                var uri = retrieveBaseUriBuilder()
                        .build()
                        .toUri();

                log.trace("Requesting VLC playback status from {}", uri);
                var vlcStatus = vlcWebClient.get()
                        .uri(uri)
                        .retrieve()
                        .bodyToMono(VlcStatus.class)
                        .block(Duration.ofMillis(750));

                log.debug("Received VLC playback status {}", vlcStatus);
                totalConnectionsRefused = 0;
                onStatusChanged(vlcStatus);
            } catch (WebClientRequestException ex) {
                handleRequestException(ex);
            } catch (WebClientException ex) {
                log.warn("Failed to retrieve VLC status, {}", ex.getMessage(), ex);
            }
        }

        private void handleRequestException(WebClientRequestException ex) {
            if (ex.getCause() instanceof ConnectException) {
                totalConnectionsRefused++;
            }

            // check if we've got more than 3 connection refused errors
            // if so, we assume that the player process was closed by the user
            if (totalConnectionsRefused >= 3) {
                log.debug("VLC connection refused, assuming the user has closed the external player");
                onStateChanged(VlcState.STOPPED);
                stop();
            }
        }

    }
}
