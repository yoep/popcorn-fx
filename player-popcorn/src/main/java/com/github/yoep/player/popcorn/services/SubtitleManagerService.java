package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.AbstractPlaybackListener;
import com.github.yoep.player.popcorn.listeners.SubtitleListener;
import com.github.yoep.popcorn.backend.adapters.video.VideoPlayback;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.PlayerStoppedEvent;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.*;
import com.github.yoep.popcorn.backend.settings.AbstractApplicationSettingsEventListener;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.subtitles.ISubtitle;
import com.github.yoep.popcorn.backend.subtitles.ISubtitleInfo;
import com.github.yoep.popcorn.backend.subtitles.ISubtitleService;
import com.github.yoep.popcorn.backend.subtitles.SubtitleInfoWrapper;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.view.services.SubtitlePickerService;
import javafx.beans.property.*;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FilenameUtils;

import java.io.File;
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
    private final ISubtitleService subtitleService;
    private final SubtitlePickerService subtitlePickerService;
    private final LocaleText localeText;
    private final EventPublisher eventPublisher;

    String quality;
    String url;
    ISubtitleInfo subtitleInfo;

    public SubtitleManagerService(ApplicationConfig applicationConfig, VideoService videoService, ISubtitleService subtitleService, SubtitlePickerService subtitlePickerService, LocaleText localeText, EventPublisher eventPublisher) {
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
     * @param info The subtitle to use.
     */
    public void updateSubtitle(ISubtitleInfo info) {
        if (info == null || info.getLanguage() == Subtitle.Language.NONE) {
            subtitleService.disableSubtitle();
        } else {
            subtitleService.updatePreferredLanguage(info.getLanguage());
        }

        onSubtitleChanged(info);
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
            public void onPlay(Player.PlayRequest request) {
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
        applicationConfig.getSettings().thenApply(ApplicationSettings::getSubtitleSettings).whenComplete((settings, throwable) -> {
            if (throwable == null) {
                subtitleSize.set(settings.getFontSize());
            } else {
                log.error("Failed to retrieve settings", throwable);
            }
        });

        applicationConfig.addListener(new AbstractApplicationSettingsEventListener() {
            @Override
            public void onSubtitleSettingsChanged(ApplicationSettings.SubtitleSettings settings) {
                subtitleSize.set(settings.getFontSize());
            }
        });
    }

    private void initializeSubtitleListener() {
        subtitleService.register(event -> {
            if (event.getEvent() == SubtitleEvent.Event.PREFERENCE_CHANGED) {
//                Optional.ofNullable(event.subtitleInfo())
//                        .ifPresent(this::onSubtitleChanged);
            }
        });
        videoService.videoPlayerProperty().addListener((observable, oldValue, newValue) -> onVideoPlayerChanged(newValue));
    }

    //endregion

    //region Functions

    private void onPlay(Player.PlayRequest request) {
        Objects.requireNonNull(request, "request cannot be null");
        this.url = request.getUrl();
        this.quality = request.getQuality();

        if (request.getSubtitle().getEnabled()) {
            subtitleService.preference().whenComplete((preference, throwable) -> {
                if (throwable == null) {
                    log.trace("Retrieved subtitle preference {}", preference);
                    if (preference.getPreference() != SubtitlePreference.Preference.DISABLED && preference.getLanguage() != Subtitle.Language.NONE) {
                        Optional.ofNullable(request.getSubtitle())
                                .filter(e -> request.hasSubtitle())
                                .map(Player.PlayRequest.PlaySubtitleRequest::getSubtitle)
                                .map(Subtitle::getInfo)
                                .map(SubtitleInfoWrapper::new)
                                .ifPresent(this::onSubtitleChanged);
                    }
                } else {
                    log.error("Failed to retrieve subtitle preference", throwable);
                }
            });
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

    private void onSubtitleDownloaded(ISubtitle subtitle) {
        Objects.requireNonNull(subtitle, "subtitle cannot be null");
        var videoPlayerOptional = videoService.getVideoPlayer();

        if (videoPlayerOptional.isEmpty())
            return;

        var videoPlayer = videoPlayerOptional.get();

        if (videoPlayer.supportsNativeSubtitleFile()) {
            log.debug("Using native subtitle file render for the current video playback");
            videoPlayer.subtitleFile(new File(subtitle.getFilePath()));
        } else {
            invokeListeners(e -> e.onSubtitleChanged(subtitle));
        }
    }

    private void onSubtitleChanged(ISubtitleInfo subtitleInfo) {
        this.subtitleInfo = subtitleInfo;

        // check if the subtitle is being disabled
        // if so, update the subtitle to none and ignore the subtitle download & parsing
        var subtitleInfoRef = new AtomicReference<>(subtitleInfo);
        subtitleService.preference().whenComplete((preference, throwable) -> {
            if (throwable == null) {
                if (preference.getPreference() == SubtitlePreference.Preference.DISABLED || preference.getLanguage() == Subtitle.Language.NONE) {
                    log.trace("Subtitle change ignore on popcorn player for {}", subtitleInfoRef.get());
                    invokeListeners(SubtitleListener::onSubtitleDisabled);
                    return;
                }

                final var imdbId = subtitleInfoRef.get().getImdbId();
                final var language = subtitleInfoRef.get().getLanguage();
                final var name = FilenameUtils.getBaseName(url);

                // check if the subtitle is a custom subtitle and doesn't contain any files yet
                // if so, pause the playback and let the user pick a custom subtitle file
                // if the custom subtitle contains files, than the passed subtitle file is from the details components
                if (subtitleInfoRef.get().isCustom()) {
                    subtitleInfoRef.set(pickCustomSubtitleTrack());

                    if (subtitleInfo == null)
                        return;
                }

                //        if (name != null && !name.isBlank()) {
                //            log.debug("Downloading subtitle \"{}\" for video playback", subtitleInfo);
                //            var matcher = SubtitleMatcher.from(name, quality);
                //
                //            var preference = subtitleService.preference();
                //            if (preference.tag() != SubtitlePreferenceTag.DISABLED) {
                //                subtitleService.downloadAndParse(subtitleInfo, matcher).whenComplete((subtitle, throwable) -> {
                //                    if (throwable == null) {
                //                        log.debug("Subtitle (imdbId: {}, language: {}) has been downloaded with success", imdbId, language);
                //                        // auto-clean the subtitle
                //                        try (subtitle) {
                //                            onSubtitleDownloaded(subtitle);
                //                        }
                //                    } else {
                //                        log.error("Video subtitle failed, " + throwable.getMessage(), throwable);
                //                        eventPublisher.publishEvent(new ErrorNotificationEvent(this, localeText.get(VideoMessage.SUBTITLE_DOWNLOAD_FILED)));
                //                    }
                //                });
                //            }
                //        }
            } else {
                log.error("Failed to retrieve subtitle preference", throwable);
            }
        });
    }

    private ISubtitleInfo pickCustomSubtitleTrack() {
        final var subtitleInfo = new AtomicReference<ISubtitleInfo>(null);

        // pause the video playback as a popup will be shown
        videoService.getVideoPlayer()
                .ifPresent(VideoPlayback::pause);

        // show the subtitle picker popup and let the user pick a subtitle file
        // if the user cancels the picking, we disable the subtitle
//        subtitlePickerService.pickCustomSubtitle().ifPresentOrElse(
//                e -> {
//                    this.subtitleInfo = SubtitleInfo.builder()
//                            .language(SubtitleLanguage.CUSTOM)
//                            .files(new SubtitleFile[]{SubtitleFile.builder()
//                                    .name("Custom")
//                                    .url(e)
//                                    .build()})
//                            .build();
//                    this.subtitleService.updatePreferredLanguage(SubtitleLanguage.CUSTOM);
//                },
//                subtitleService::disableSubtitle
//        );

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
