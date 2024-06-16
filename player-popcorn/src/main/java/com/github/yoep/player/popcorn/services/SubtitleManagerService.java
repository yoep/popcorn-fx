package com.github.yoep.player.popcorn.services;

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
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import com.github.yoep.popcorn.backend.subtitles.Subtitle;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.model.*;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.view.services.SubtitlePickerService;
import javafx.beans.property.*;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FilenameUtils;

import java.util.Objects;
import java.util.Optional;
import java.util.Queue;
import java.util.concurrent.ConcurrentLinkedDeque;
import java.util.concurrent.atomic.AtomicReference;
import java.util.function.Consumer;

@Slf4j
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

    String quality;
    String url;
    SubtitleInfo subtitleInfo;

    public SubtitleManagerService(ApplicationConfig applicationConfig, VideoService videoService, SubtitleService subtitleService, SubtitlePickerService subtitlePickerService, LocaleText localeText, EventPublisher eventPublisher) {
        this.applicationConfig = applicationConfig;
        this.videoService = videoService;
        this.subtitleService = subtitleService;
        this.subtitlePickerService = subtitlePickerService;
        this.localeText = localeText;
        this.eventPublisher = eventPublisher;
        init();
    }

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
    public void updateSubtitle(SubtitleInfo subtitleInfo) {
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

    private void init() {
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
            if (event.tag() == SubtitleEventTag.SubtitleInfoChanged) {
                Optional.ofNullable(event.subtitleInfo())
                        .ifPresent(this::onSubtitleChanged);
            }
        });
        videoService.videoPlayerProperty().addListener((observable, oldValue, newValue) -> onVideoPlayerChanged(newValue));
    }

    //endregion

    //region Functions

    private void onPlay(PlayRequest request) {
        Objects.requireNonNull(request, "request cannot be null");
        this.url = request.getUrl();
        this.quality = request.getQuality().orElse(null);

        if (request.isSubtitlesEnabled()) {
            var preference = subtitleService.preference();

            log.trace("Retrieved subtitle preference {}", preference);
            var tag = preference.tag();
            if (tag != SubtitlePreferenceTag.DISABLED && preference.language() != SubtitleLanguage.NONE) {
                request.getSubtitleInfo()
                        .ifPresent(this::onSubtitleChanged);
            }
        } else {
            invokeListeners(SubtitleListener::onSubtitleDisabled);
        }
    }

    private void onVideoPlayerChanged(VideoPlayback newPlayer) {
        if (newPlayer == null)
            return;

        // invoke the subtitle changed again for the new player
        Optional.ofNullable(subtitleInfo)
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
        this.subtitleInfo = subtitleInfo;

        // check if the subtitle is being disabled
        // if so, update the subtitle to none and ignore the subtitle download & parsing
        if (subtitleService.isDisabled() || subtitleInfo == null || subtitleInfo.isNone()) {
            log.trace("Subtitle change ignore on popcorn player for {}", subtitleInfo);
            invokeListeners(SubtitleListener::onSubtitleDisabled);
            return;
        }

        final var imdbId = subtitleInfo.imdbId();
        final var language = subtitleInfo.language();
        final var name = FilenameUtils.getBaseName(url);

        // check if the subtitle is a custom subtitle and doesn't contain any files yet
        // if so, pause the playback and let the user pick a custom subtitle file
        // if the custom subtitle contains files, than the passed subtitle file is from the details components
        if (subtitleInfo.isCustom()) {
            subtitleInfo = pickCustomSubtitleTrack();

            if (subtitleInfo == null)
                return;
        }

        if (name != null && !name.isBlank()) {
            log.debug("Downloading subtitle \"{}\" for video playback", subtitleInfo);
            var matcher = SubtitleMatcher.from(name, quality);

            var preference = subtitleService.preference();
            if (preference.tag() != SubtitlePreferenceTag.DISABLED) {
                subtitleService.downloadAndParse(subtitleInfo, matcher).whenComplete((subtitle, throwable) -> {
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
                });
            }
        }
    }

    private SubtitleInfo pickCustomSubtitleTrack() {
        final var subtitleInfo = new AtomicReference<SubtitleInfo>(null);

        // pause the video playback as a popup will be shown
        videoService.getVideoPlayer()
                .ifPresent(VideoPlayback::pause);

        // show the subtitle picker popup and let the user pick a subtitle file
        // if the user cancels the picking, we disable the subtitle
        subtitlePickerService.pickCustomSubtitle().ifPresentOrElse(
                e -> {
                    this.subtitleInfo = SubtitleInfo.builder()
                            .language(SubtitleLanguage.CUSTOM)
                            .files(new SubtitleFile[]{SubtitleFile.builder()
                                    .name("Custom")
                                    .url(e)
                                    .build()})
                            .build();
                    this.subtitleService.updatePreferredLanguage(SubtitleLanguage.CUSTOM);
                },
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
