package com.github.yoep.popcorn.backend.subtitles;

import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.backend.media.providers.models.Movie;
import com.github.yoep.popcorn.backend.media.providers.models.Show;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleMatcher;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleType;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.ReadOnlyObjectProperty;
import javafx.beans.property.SimpleObjectProperty;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.scheduling.annotation.Async;
import org.springframework.util.Assert;

import java.io.ByteArrayInputStream;
import java.io.File;
import java.io.InputStream;
import java.nio.charset.Charset;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import java.util.stream.Stream;

@Slf4j
@RequiredArgsConstructor
public class SubtitleServiceImpl implements SubtitleService {
    public static final String SUBTITLE_PROPERTY = "activeSubtitle";

    private final ObjectProperty<Subtitle> activeSubtitle = new SimpleObjectProperty<>(this, SUBTITLE_PROPERTY, null);
    private final SubtitleDelegate delegate;

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
    public CompletableFuture<List<SubtitleInfo>> retrieveSubtitles(final Movie media) {
        Assert.notNull(media, "media cannot be null");
        return CompletableFuture.completedFuture(
                Stream.concat(delegate.defaultOptions().stream(), delegate.subtitles(media).stream()).toList());
    }

    @Override
    @Async
    public CompletableFuture<List<SubtitleInfo>> retrieveSubtitles(final Show media, final Episode episode) {
        Assert.notNull(media, "media cannot be null");
        Assert.notNull(episode, "episode cannot be null");
        return CompletableFuture.completedFuture(
                Stream.concat(delegate.defaultOptions().stream(), delegate.subtitles(media, episode).stream()).toList());
    }

    @Override
    @Async
    public CompletableFuture<List<SubtitleInfo>> retrieveSubtitles(final String filename) {
        Assert.hasText(filename, "filename cannot be empty");
        return CompletableFuture.completedFuture(
                Stream.concat(delegate.defaultOptions().stream(), delegate.subtitles(filename).stream()).toList());
    }

    @Override
    @Async
    public CompletableFuture<Subtitle> parse(File file, Charset encoding) {
        Assert.notNull(file, "file cannot be null");
        return CompletableFuture.completedFuture(delegate.parse(file.getAbsolutePath()));
    }

    @Override
    public CompletableFuture<Subtitle> downloadAndParse(SubtitleInfo subtitleInfo, SubtitleMatcher matcher) {
        Objects.requireNonNull(subtitleInfo, "subtitleInfo cannot be null");
        Objects.requireNonNull(matcher, "matcher cannot be null");
        return CompletableFuture.completedFuture(delegate.download(subtitleInfo, matcher));
    }

    @Override
    public InputStream convert(Subtitle subtitle, SubtitleType type) {
        Objects.requireNonNull(subtitle, "subtitle cannot be null");
        var output = delegate.convert(subtitle, type);
        log.info("Convert to {}", output);

        return new ByteArrayInputStream(output.getBytes());
    }

    @Override
    public SubtitleInfo getDefaultOrInterfaceLanguage(List<SubtitleInfo> subtitles) {
        Assert.notNull(subtitles, "subtitles cannot be null");
        return delegate.getDefaultOrInterfaceLanguage(subtitles);
    }

    //endregion
}
