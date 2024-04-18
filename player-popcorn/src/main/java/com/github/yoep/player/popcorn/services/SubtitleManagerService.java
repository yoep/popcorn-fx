package com.github.yoep.player.popcorn.services;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.player.popcorn.listeners.AbstractPlaybackListener;
import com.github.yoep.player.popcorn.listeners.SubtitleListener;
import com.github.yoep.player.popcorn.messages.VideoMessage;
import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.adapters.video.VideoPlayback;
import com.github.yoep.popcorn.backend.events.ErrorNotificationEvent;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.PlayerStoppedEvent;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.ApplicationConfigEvent;
import com.github.yoep.popcorn.backend.subtitles.Subtitle;
import com.github.yoep.popcorn.backend.subtitles.SubtitleEvent;
import com.github.yoep.popcorn.backend.subtitles.SubtitlePickerService;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleMatcher;
import javafx.beans.property.*;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FilenameUtils;
import org.springframework.lang.Nullable;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;
import java.util.Objects;
import java.util.Optional;
import java.util.Queue;
import java.util.concurrent.ConcurrentLinkedDeque;
import java.util.concurrent.atomic.AtomicReference;
import java.util.function.Consumer;

@Slf4j
@Service
@RequiredArgsConstructor
public class SubtitleManagerService {
    public static final String SUBTITLE_SIZE_PROPERTY = "subtitleSize";
    public static final String SUBTITLE_OFFSET_PROPERTY = "subtitleOffset";

    private final IntegerProperty subtitleSize = new SimpleIntegerProperty(this, SUBTITLE_SIZE_PROPERTY);
    private final LongProperty subtitleOffset = new SimpleLongProperty(this, SUBTITLE_OFFSET_PROPERTY);
    private final Queue<SubtitleListener> listeners = new ConcurrentLinkedDeque<>();
    private final ApplicationConfig applicationConfig;
    private final VideoService videoService;
    private final SubtitleService subtitleService;
    private final SubtitlePickerService subtitlePickerService;
    private final LocaleText localeText;
    private final EventPublisher eventPublisher;

    private String quality;
    private String url;

    //region Properties

    /**
     * Get the subtitle size property.
     *
     * @return Returns the subtitle size property.
     */
    public ReadOnlyIntegerProperty subtitleSizeProperty() {
        return subtitleSize;
    }

    public int getSubtitleSize() {
        return subtitleSize.get();
    }

    public void setSubtitleSize(int subtitleSize) {
        this.subtitleSize.set(subtitleSize);
    }

    public long getSubtitleOffset() {
        return subtitleOffset.get();
    }

    //endregion

    //region Methods

    /**
     * Update the subtitle offset for the current playback.
     *
     * @param offset The offset to apply to the subtitle.
     */
    public void updateSubtitleOffset(int offset) {
        subtitleOffset.set(offset);
        var videoPlayer = videoService.getVideoPlayer();

        if (videoPlayer.isEmpty()) {
            return;
        }

        if (videoPlayer.get().supportsNativeSubtitleFile()) {
            videoPlayer.get().subtitleDelay(offset);
        }
    }

    /**
     * Update the active subtitle.
     * When the given subtitle is {@code null}, then the subtitle track will be disabled.
     *
     * @param subtitleInfo The subtitle to use.
     */
    public void updateSubtitle(@Nullable SubtitleInfo subtitleInfo) {
        if (subtitleInfo == null || subtitleInfo.isNone()) {
            subtitleService.disableSubtitle();
        } else {
            subtitleService.updateSubtitle(subtitleInfo);
        }

        onSubtitleChanged(subtitleInfo);
    }

    public void registerListener(SubtitleListener listener) {
        Objects.requireNonNull(listener, "listener cannot be null");
        listeners.add(listener);
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    void init() {
        log.trace("Initializing video player subtitle service");
        initializeSubtitleSize();
        initializeSubtitleListener();
        videoService.addListener(new AbstractPlaybackListener() {
            @Override
            public void onPlay(PlayRequest request) {
                SubtitleManagerService.this.onPlay(request);
            }
        });
        eventPublisher.register(PlayerStoppedEvent.class, event -> {
            this.url = null;
            this.quality = null;
            updateSubtitleOffset(0);
            return event;
        });
    }

    private void initializeSubtitleSize() {
        var subtitleSettings = applicationConfig.getSettings().getSubtitleSettings();

        subtitleSize.set(subtitleSettings.getFontSize());
        applicationConfig.register(event -> {
            if (event.tag == ApplicationConfigEvent.Tag.SUBTITLE_SETTINGS_CHANGED) {
                subtitleSize.set(event.getUnion().getSubtitleSettingsChanged_body().getSettings().getFontSize());
            }
        });
    }

    private void initializeSubtitleListener() {
        subtitleService.register(event -> {
            if (event.getTag() == SubtitleEvent.Tag.SubtitleInfoChanged) {
                Optional.ofNullable(event.getUnion())
                        .map(SubtitleEvent.SubtitleEventCUnion::getSubtitle_info_changed)
                        .map(SubtitleEvent.SubtitleInfoChanged_Body::getSubtitleInfo)
                        .ifPresent(this::onSubtitleChanged);
            }
        });
        videoService.videoPlayerProperty().addListener((observable, oldValue, newValue) -> onVideoPlayerChanged(newValue));
    }

    //endregion

    //region Functions

    private void onPlay(PlayRequest request) {
        if (request.isSubtitlesEnabled()) {
            subtitleService.preferredSubtitle()
                    .filter(e -> !e.isNone())
                    .ifPresent(this::onSubtitleChanged);
        } else {
            invokeListeners(SubtitleListener::onSubtitleDisabled);
        }
    }

    private void onVideoPlayerChanged(VideoPlayback newPlayer) {
        if (newPlayer == null)
            return;

        // invoke the subtitle changed again for the new player
        subtitleService.preferredSubtitle()
                .filter(e -> !e.isNone())
                .ifPresent(this::onSubtitleChanged);
    }

    private void onSubtitleDownloaded(Subtitle subtitle) {
        Objects.requireNonNull(subtitle, "subtitle cannot be null");
        var videoPlayerOptional = videoService.getVideoPlayer();

        if (videoPlayerOptional.isEmpty())
            return;

        var videoPlayer = videoPlayerOptional.get();

        if (videoPlayer.supportsNativeSubtitleFile()) {
            log.debug("Using native subtitle file render for the current video playback");
            videoPlayer.subtitleFile(subtitle.getFile());
        } else {
            invokeListeners(e -> e.onSubtitleChanged(subtitle));
        }
    }

    private void onSubtitleChanged(SubtitleInfo subtitleInfo) {
        // check if the subtitle is being disabled
        // if so, update the subtitle to none and ignore the subtitle download & parsing
        if (subtitleService.isDisabled() || subtitleInfo == null || subtitleInfo.isNone()) {
            log.trace("Subtitle change ignore on popcorn player for {}", subtitleInfo);
            invokeListeners(SubtitleListener::onSubtitleDisabled);
            return;
        }

        final var imdbId = subtitleInfo.getImdbId();
        final var language = subtitleInfo.getLanguage();

        // check if the subtitle is a custom subtitle and doesn't contain any files yet
        // if so, pause the playback and let the user pick a custom subtitle file
        // if the custom subtitle contains files, than the passed subtitle file is from the details components
        if (subtitleInfo.isCustom()) {
            subtitleInfo = pickCustomSubtitleTrack();

            if (subtitleInfo == null)
                return;
        }

        log.debug("Downloading subtitle \"{}\" for video playback", subtitleInfo);
        var matcher = SubtitleMatcher.from(FilenameUtils.getBaseName(url), quality);

        subtitleService.preferredSubtitle()
                .ifPresent(preferredSubtitle -> subtitleService.downloadAndParse(preferredSubtitle, matcher).whenComplete((subtitle, throwable) -> {
                    if (throwable == null) {
                        log.debug("Subtitle (imdbId: {}, language: {}) has been downloaded with success", imdbId, language);
                        // auto-clean the subtitle
                        try (subtitle) {
                            onSubtitleDownloaded(subtitle);
                        }
                    } else {
                        log.error("Video subtitle failed, " + throwable.getMessage(), throwable);
                        eventPublisher.publishEvent(new ErrorNotificationEvent(this, localeText.get(VideoMessage.SUBTITLE_DOWNLOAD_FILED)));
                    }

                    preferredSubtitle.close();
                }));
    }

    private SubtitleInfo pickCustomSubtitleTrack() {
        final var subtitleInfo = new AtomicReference<SubtitleInfo>(null);

        // pause the video playback as a popup will be shown
        videoService.getVideoPlayer()
                .ifPresent(VideoPlayback::pause);

        // show the subtitle picker popup and let the user pick a subtitle file
        // if the user cancels the picking, we disable the subtitle
        subtitlePickerService.pickCustomSubtitle().ifPresentOrElse(
                subtitleService::updateCustomSubtitle,
                subtitleService::disableSubtitle
        );

        // resume the video playback
        videoService.getVideoPlayer()
                .ifPresent(VideoPlayback::resume);

        return subtitleInfo.get();
    }

    private void invokeListeners(Consumer<SubtitleListener> action) {
        listeners.forEach(e -> {
            try {
                action.accept(e);
            } catch (Exception ex) {
                log.error("Failed to invoke listener, {}", ex.getMessage(), ex);
            }
        });
    }

    //endregion
}
