package com.github.yoep.popcorn.backend.subtitles;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFxInstance;
import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.backend.media.providers.models.MovieDetails;
import com.github.yoep.popcorn.backend.media.providers.models.ShowDetails;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfoSet;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleMatcher;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleType;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.ReadOnlyObjectProperty;
import javafx.beans.property.SimpleObjectProperty;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.scheduling.annotation.Async;
import org.springframework.stereotype.Service;
import org.springframework.util.Assert;

import java.io.File;
import java.nio.charset.Charset;
import java.util.Collections;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import java.util.stream.Collectors;
import java.util.stream.Stream;

@Slf4j
@Service
@RequiredArgsConstructor
public class SubtitleServiceImpl implements SubtitleService {
    public static final String SUBTITLE_PROPERTY = "activeSubtitle";

    private final ObjectProperty<Subtitle> activeSubtitle = new SimpleObjectProperty<>(this, SUBTITLE_PROPERTY, null);
    private final Object mutex = new Object();

    //region Properties

    @Override
    public Optional<Subtitle> getActiveSubtitle() {
        return Optional.ofNullable(activeSubtitle.get());
    }

    @Override
    public ReadOnlyObjectProperty<Subtitle> activeSubtitleProperty() {
        return activeSubtitle;
    }

    @Override
    public void setActiveSubtitle(Subtitle activeSubtitle) {
        this.activeSubtitle.set(activeSubtitle);

        updateSubtitle(Optional.ofNullable(activeSubtitle)
                .flatMap(Subtitle::getSubtitleInfo)
                .orElse(null));
    }

    @Override
    public boolean isDisabled() {
        return FxLib.INSTANCE.is_subtitle_disabled(PopcornFxInstance.INSTANCE.get()) == 1;
    }

    //endregion

    //region Methods

    @Override
    @Async
    public CompletableFuture<List<SubtitleInfo>> retrieveSubtitles(final MovieDetails media) {
        Objects.requireNonNull(media, "media cannot be null");
        var subtitles = Optional.ofNullable(FxLib.INSTANCE.movie_subtitles(PopcornFxInstance.INSTANCE.get(), media))
                .map(SubtitleInfoSet::getSubtitles)
                .orElse(Collections.emptyList());

        log.debug("Retrieved movie subtitles {}", subtitles);
        return CompletableFuture.completedFuture(
                Stream.concat(defaultOptions().stream(), subtitles.stream()).toList());
    }

    @Override
    @Async
    public CompletableFuture<List<SubtitleInfo>> retrieveSubtitles(final ShowDetails media, final Episode episode) {
        Objects.requireNonNull(media, "media cannot be null");
        Objects.requireNonNull(episode, "episode cannot be null");
        var subtitles = Optional.ofNullable(FxLib.INSTANCE.episode_subtitles(PopcornFxInstance.INSTANCE.get(), media, episode))
                .map(SubtitleInfoSet::getSubtitles)
                .orElse(Collections.emptyList());

        log.debug("Retrieved episode subtitle {}", subtitles);
        return CompletableFuture.completedFuture(
                Stream.concat(defaultOptions().stream(), subtitles.stream()).toList());
    }

    @Override
    @Async
    public CompletableFuture<List<SubtitleInfo>> retrieveSubtitles(final String filename) {
        Assert.hasText(filename, "filename cannot be empty");
        var subtitles = Optional.ofNullable(FxLib.INSTANCE.filename_subtitles(PopcornFxInstance.INSTANCE.get(), filename))
                .map(SubtitleInfoSet::getSubtitles)
                .orElse(Collections.emptyList());

        return CompletableFuture.completedFuture(
                Stream.concat(defaultOptions().stream(), subtitles.stream()).toList());
    }

    @Override
    @Async
    public CompletableFuture<Subtitle> parse(File file, Charset encoding) {
        Objects.requireNonNull(file, "file cannot be null");
        synchronized (mutex) {
            return CompletableFuture.completedFuture(FxLib.INSTANCE.parse_subtitle(PopcornFxInstance.INSTANCE.get(), file.getAbsolutePath()));
        }
    }

    @Override
    public CompletableFuture<String> download(SubtitleInfo subtitleInfo, SubtitleMatcher matcher) {
        Objects.requireNonNull(subtitleInfo, "subtitleInfo cannot be null");
        Objects.requireNonNull(matcher, "matcher cannot be null");
        synchronized (mutex) {
            log.debug("Starting subtitle download subtitleInfo: {}, matcher: {}", subtitleInfo, matcher);
            return CompletableFuture.completedFuture(FxLib.INSTANCE.download(PopcornFxInstance.INSTANCE.get(), subtitleInfo, matcher));
        }
    }

    @Override
    public CompletableFuture<Subtitle> downloadAndParse(SubtitleInfo subtitleInfo, SubtitleMatcher matcher) {
        Objects.requireNonNull(subtitleInfo, "subtitleInfo cannot be null");
        Objects.requireNonNull(matcher, "matcher cannot be null");
        synchronized (mutex) {
            log.debug("Starting subtitle download subtitleInfo: {}, matcher: {}", subtitleInfo, matcher);
            var subtitle = FxLib.INSTANCE.download_and_parse_subtitle(PopcornFxInstance.INSTANCE.get(), subtitleInfo, matcher);
            log.info("Downloaded and parsed subtitle info {} to {}", subtitleInfo, subtitle.getFilepath());
            return CompletableFuture.completedFuture(subtitle);
        }
    }

    @Override
    public SubtitleInfo getDefaultOrInterfaceLanguage(List<SubtitleInfo> subtitles) {
        Objects.requireNonNull(subtitles, "subtitles cannot be null");
        subtitles = subtitles.stream()
                .filter(e -> !e.isSpecial())
                .collect(Collectors.toList());

        var count = subtitles.size();
        var array = (SubtitleInfo[]) new SubtitleInfo().toArray(count);

        for (int i = 0; i < count; i++) {
            var subtitle = subtitles.get(i);
            array[i].imdbId = subtitle.imdbId;
            array[i].language = subtitle.language;
            array[i].files = subtitle.files;
            array[i].len = subtitle.len;
        }

        synchronized (mutex) {
            return FxLib.INSTANCE.select_or_default_subtitle(PopcornFxInstance.INSTANCE.get(), array, count);
        }
    }

    @Override
    public String serve(Subtitle subtitle, SubtitleType type) {
        Objects.requireNonNull(subtitle, "subtitle cannot be null");
        synchronized (mutex) {
            return FxLib.INSTANCE.serve_subtitle(PopcornFxInstance.INSTANCE.get(), subtitle, type.ordinal());
        }
    }

    @Override
    public Optional<SubtitleInfo> preferredSubtitle() {
        synchronized (mutex) {
            return Optional.ofNullable(FxLib.INSTANCE.retrieve_preferred_subtitle(PopcornFxInstance.INSTANCE.get()));
        }
    }

    @Override
    public SubtitleLanguage preferredSubtitleLanguage() {
        synchronized (mutex) {
            return FxLib.INSTANCE.retrieve_preferred_subtitle_language(PopcornFxInstance.INSTANCE.get());
        }
    }

    @Override
    public void updateSubtitle(SubtitleInfo subtitle) {
        synchronized (mutex) {
            if (subtitle != null) {
                log.trace("Updating subtitle to {}", subtitle);
                FxLib.INSTANCE.update_subtitle(PopcornFxInstance.INSTANCE.get(), subtitle);
            } else {
                log.trace("Clearing the preferred subtitle");
                FxLib.INSTANCE.reset_subtitle(PopcornFxInstance.INSTANCE.get());
            }
        }
    }

    @Override
    public void updateCustomSubtitle(String subtitleFilepath) {
        Objects.requireNonNull(subtitleFilepath, "subtitleFilepath cannot be null");
        synchronized (mutex) {
            log.trace("Updating subtitle custom filepath to {}", subtitleFilepath);
            FxLib.INSTANCE.update_subtitle_custom_file(PopcornFxInstance.INSTANCE.get(), subtitleFilepath);
        }
    }

    @Override
    public void disableSubtitle() {
        FxLib.INSTANCE.disable_subtitle(PopcornFxInstance.INSTANCE.get());
    }

    //endregion

    private static List<SubtitleInfo> defaultOptions() {
        return FxLib.INSTANCE.default_subtitle_options(PopcornFxInstance.INSTANCE.get()).getSubtitles();
    }
}