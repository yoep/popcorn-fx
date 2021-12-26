package com.github.yoep.player.popcorn.subtitles;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.player.popcorn.services.VideoService;
import com.github.yoep.popcorn.backend.adapters.video.VideoPlayer;
import com.github.yoep.popcorn.backend.events.ErrorNotificationEvent;
import com.github.yoep.popcorn.backend.events.PlayMediaEvent;
import com.github.yoep.popcorn.backend.events.PlayVideoEvent;
import com.github.yoep.popcorn.backend.events.PlayerStoppedEvent;
import com.github.yoep.popcorn.backend.messages.VideoMessage;
import com.github.yoep.popcorn.backend.settings.SettingsService;
import com.github.yoep.popcorn.backend.subtitles.Subtitle;
import com.github.yoep.popcorn.backend.subtitles.SubtitlePickerService;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.models.SubtitleInfo;
import com.github.yoep.popcorn.backend.subtitles.models.SubtitleMatcher;
import javafx.beans.property.IntegerProperty;
import javafx.beans.property.SimpleIntegerProperty;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FilenameUtils;
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.context.event.EventListener;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;

@Slf4j
@Service
@RequiredArgsConstructor
public class PopcornSubtitleService {
    public static final String SUBTITLE_SIZE_PROPERTY = "subtitleSize";

    private final IntegerProperty subtitleSize = new SimpleIntegerProperty(this, SUBTITLE_SIZE_PROPERTY);
    private final SettingsService settingsService;
    private final VideoService videoService;
    private final SubtitleService subtitleService;
    private final SubtitlePickerService subtitlePickerService;
    private final LocaleText localeText;
    private final ApplicationEventPublisher eventPublisher;

    private String quality;
    private String url;

    //region Properties

    /**
     * Get the subtitle font size.
     *
     * @return Returns the subtitle font size.
     */
    public int getSubtitleSize() {
        return subtitleSize.get();
    }

    /**
     * Get the subtitle size property.
     *
     * @return Returns the subtitle size property.
     */
    public IntegerProperty subtitleSizeProperty() {
        return subtitleSize;
    }

    /**
     * Set the new subtitle size.
     *
     * @param subtitleSize The new subtitle size.
     */
    public void setSubtitleSize(int subtitleSize) {
        this.subtitleSize.set(subtitleSize);
    }

    /**
     * Set the subtitle offset for the video player.
     *
     * @param offset The subtitle offset.
     */
    public void setSubtitleOffset(long offset) {
        var videoPlayer = videoService.getVideoPlayer();

        if (videoPlayer.isEmpty()) {
            return;
        }

        if (videoPlayer.get().supportsNativeSubtitleFile()) {
            videoPlayer.get().subtitleDelay(offset);
        } else {
            log.trace("Video player does not support native subtitle files, ignoring subtitle offset update");
        }
    }

    //endregion

    //region Methods

    public void setSubtitle(SubtitleInfo subtitleInfo) {
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
    private void init() {
        log.trace("Initializing video player subtitle service");
        initializeSubtitleSize();
        initializeSubtitleListener();
    }

    private void initializeSubtitleSize() {
        subtitleSize.set(settingsService.getSettings().getSubtitleSettings().getFontSize());
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
                    .ifPresent(VideoPlayer::pause);

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
                    .ifPresent(VideoPlayer::resume);
        }

        log.debug("Downloading subtitle \"{}\" for video playback", subtitleInfo);
        var matcher = SubtitleMatcher.from(FilenameUtils.getBaseName(url), quality);

        subtitleService.downloadAndParse(subtitleInfo, matcher).whenComplete((subtitles, throwable) -> {
            if (throwable == null) {
                log.debug("Subtitle (imdbId: {}, language: {}) has been downloaded with success", imdbId, language);
                subtitleService.setActiveSubtitle(subtitles);

                onSubtitleDownloaded(subtitles);
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
