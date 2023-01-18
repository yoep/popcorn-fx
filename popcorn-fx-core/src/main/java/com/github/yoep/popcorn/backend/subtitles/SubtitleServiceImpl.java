package com.github.yoep.popcorn.backend.subtitles;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFxInstance;
import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.backend.media.providers.models.MovieDetails;
import com.github.yoep.popcorn.backend.media.providers.models.ShowDetails;
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

import java.io.ByteArrayInputStream;
import java.io.File;
import java.io.InputStream;
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
    }

    //endregion

    //region Methods

    @Override
    @Async
    public CompletableFuture<List<SubtitleInfo>> retrieveSubtitles(final MovieDetails media) {
        Assert.notNull(media, "media cannot be null");
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
        Assert.notNull(media, "media cannot be null");
        Assert.notNull(episode, "episode cannot be null");
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
        Assert.notNull(file, "file cannot be null");
        return CompletableFuture.completedFuture(FxLib.INSTANCE.parse_subtitle(PopcornFxInstance.INSTANCE.get(), file.getAbsolutePath()));
    }

    @Override
    public CompletableFuture<Subtitle> downloadAndParse(SubtitleInfo subtitleInfo, SubtitleMatcher matcher) {
        Objects.requireNonNull(subtitleInfo, "subtitleInfo cannot be null");
        Objects.requireNonNull(matcher, "matcher cannot be null");
        synchronized (mutex) {
            return CompletableFuture.completedFuture(FxLib.INSTANCE.download_subtitle(PopcornFxInstance.INSTANCE.get(), subtitleInfo, matcher));
        }
    }

    @Override
    public InputStream convert(Subtitle subtitle, SubtitleType type) {
        Objects.requireNonNull(subtitle, "subtitle cannot be null");
        var subtitleType = (int) type.toNative();
        var output = FxLib.INSTANCE.subtitle_to_raw(PopcornFxInstance.INSTANCE.get(), subtitle, subtitleType);

        return new ByteArrayInputStream(output.getBytes());
    }

    @Override
    public SubtitleInfo getDefaultOrInterfaceLanguage(List<SubtitleInfo> subtitles) {
        Assert.notNull(subtitles, "subtitles cannot be null");
        subtitles = subtitles.stream()
                .filter(e -> !e.isSpecial())
                .collect(Collectors.toList());

        var count = subtitles.size();
        var array = (SubtitleInfo[]) new SubtitleInfo().toArray(count);

        for (int i = 0; i < count; i++) {
            var subtitle = subtitles.get(i);
            array[i].imdbId = subtitle.imdbId;
            array[i].language = subtitle.language;
            array[i].infoPointer = subtitle.infoPointer;
        }

        return FxLib.INSTANCE.select_or_default_subtitle(PopcornFxInstance.INSTANCE.get(), array, count);
    }

    //endregion

    private static List<SubtitleInfo> defaultOptions() {
        return FxLib.INSTANCE.default_subtitle_options(PopcornFxInstance.INSTANCE.get()).getSubtitles();
    }
}
