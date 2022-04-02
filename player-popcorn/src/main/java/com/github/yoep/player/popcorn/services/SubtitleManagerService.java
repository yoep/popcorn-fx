package com.github.yoep.player.popcorn.services;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.player.popcorn.messages.VideoMessage;
import com.github.yoep.popcorn.backend.adapters.video.VideoPlayback;
import com.github.yoep.popcorn.backend.events.ErrorNotificationEvent;
import com.github.yoep.popcorn.backend.events.PlayMediaEvent;
import com.github.yoep.popcorn.backend.events.PlayVideoEvent;
import com.github.yoep.popcorn.backend.events.PlayerStoppedEvent;
import com.github.yoep.popcorn.backend.settings.SettingsService;
import com.github.yoep.popcorn.backend.settings.models.SubtitleSettings;
import com.github.yoep.popcorn.backend.subtitles.Subtitle;
import com.github.yoep.popcorn.backend.subtitles.SubtitlePickerService;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.models.SubtitleInfo;
import com.github.yoep.popcorn.backend.subtitles.models.SubtitleMatcher;
import javafx.beans.property.*;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FilenameUtils;
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.context.event.EventListener;
import org.springframework.lang.Nullable;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;
import java.util.Objects;

@Slf4j
@Service
@RequiredArgsConstructor
public class SubtitleManagerService {
    public static final String SUBTITLE_SIZE_PROPERTY = "subtitleSize";
    public static final String SUBTITLE_OFFSET_PROPERTY = "subtitleOffset";

    private final IntegerProperty subtitleSize = new SimpleIntegerProperty(this, SUBTITLE_SIZE_PROPERTY);
    private final LongProperty subtitleOffset = new SimpleLongProperty(this, SUBTITLE_OFFSET_PROPERTY);
    private final SettingsService settingsService;
    private final VideoService videoService;
    private final SubtitleService subtitleService;
    private final SubtitlePickerService subtitlePickerService;
    private final LocaleText localeText;
    private final ApplicationEventPublisher eventPublisher;

    private String quality;
    private String url;

    //region Properties

    public ReadOnlyObjectProperty<Subtitle> activeSubtitleProperty() {
        return subtitleService.activeSubtitleProperty();
    }

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
        onSubtitleChanged(subtitleInfo);
    }

    @EventListener
    public void onPlayVideo(PlayVideoEvent activity) {
        this.url = activity.getUrl();
    }

    @EventListener
    public void onPlayMedia(PlayMediaEvent activity) {
        this.quality = activity.getQuality();
    }

    @EventListener(PlayerStoppedEvent.class)
    public void onPlayerStopped() {
        this.url = null;
        this.quality = null;
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    void init() {
        log.trace("Initializing video player subtitle service");
        initializeSubtitleSize();
        initializeSubtitleListener();
    }

    private void initializeSubtitleSize() {
        var subtitleSettings = settingsService.getSettings().getSubtitleSettings();

        subtitleSize.set(subtitleSettings.getFontSize());
        subtitleSettings.addListener(evt -> {
            if (Objects.equals(evt.getPropertyName(), SubtitleSettings.FONT_SIZE_PROPERTY)) {
                subtitleSize.set((Integer) evt.getNewValue());
            }
        });
    }

    private void initializeSubtitleListener() {
        subtitleService.activeSubtitleProperty().addListener((observable, oldValue, newValue) ->
                newValue.getSubtitleInfo().ifPresent(this::onSubtitleChanged));
    }

    //endregion

    //region Functions

    private void onSubtitleDownloaded(Subtitle subtitle) {
        var videoPlayerOptional = videoService.getVideoPlayer();

        if (videoPlayerOptional.isEmpty())
            return;

        var videoPlayer = videoPlayerOptional.get();

        if (videoPlayer.supportsNativeSubtitleFile()) {
            log.debug("Using native subtitle file render for the current video playback");
            videoPlayer.subtitleFile(subtitle.getFile());
        }
    }

    private void onSubtitleChanged(SubtitleInfo subtitleInfo) {
        // check if the subtitle is being disabled
        // if so, update the subtitle to none and ignore the subtitle download & parsing
        if (subtitleInfo == null || subtitleInfo.isNone()) {
            disableSubtitleTrack();
            return;
        }

        final var imdbId = subtitleInfo.getImdbId();
        final var language = subtitleInfo.getLanguage();

        // check if the subtitle is a custom subtitle and doesn't contain any files yet
        // if so, pause the playback and let the user pick a custom subtitle file
        // if the custom subtitle contains files, than the passed subtitle file is from the details components
        if (subtitleInfo.isCustom() && subtitleInfo.getFiles().isEmpty()) {
            // pause the video playback as a popup will be shown
            videoService.getVideoPlayer()
                    .ifPresent(VideoPlayback::pause);

            // show the subtitle picker popup
            var customSubtitle = subtitlePickerService.pickCustomSubtitle();

            if (customSubtitle.isPresent()) {
                // overrule the given subtitleInfo with the custom subtitle file picked by the user
                subtitleInfo = customSubtitle.get();
            } else {
                disableSubtitleTrack();
                return;
            }

            // resume the video playback
            videoService.getVideoPlayer()
                    .ifPresent(VideoPlayback::resume);
        }

        log.debug("Downloading subtitle \"{}\" for video playback", subtitleInfo);
        var matcher = SubtitleMatcher.from(FilenameUtils.getBaseName(url), quality);

        subtitleService.downloadAndParse(subtitleInfo, matcher).whenComplete((subtitle, throwable) -> {
            if (throwable == null) {
                log.debug("Subtitle (imdbId: {}, language: {}) has been downloaded with success", imdbId, language);
                subtitleService.setActiveSubtitle(subtitle);

                onSubtitleDownloaded(subtitle);
            } else {
                log.error("Video subtitle failed, " + throwable.getMessage(), throwable);
                eventPublisher.publishEvent(new ErrorNotificationEvent(this, localeText.get(VideoMessage.SUBTITLE_DOWNLOAD_FILED)));
            }
        });
    }

    private void disableSubtitleTrack() {
        log.debug("Disabling the subtitle track for the video playback");
        subtitleService.setActiveSubtitle(Subtitle.none());
    }

    //endregion
}
