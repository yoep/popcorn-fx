package com.github.yoep.popcorn.backend.subtitles;

import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.backend.media.providers.models.Movie;
import com.github.yoep.popcorn.backend.media.providers.models.Show;
import com.github.yoep.popcorn.backend.settings.SettingsService;
import com.github.yoep.popcorn.backend.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.models.SubtitleSettings;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleMatcher;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleType;
import com.github.yoep.popcorn.backend.subtitles.parser.Parser;
import com.github.yoep.popcorn.backend.subtitles.parser.SrtParser;
import com.github.yoep.popcorn.backend.subtitles.parser.VttParser;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.ReadOnlyObjectProperty;
import javafx.beans.property.SimpleObjectProperty;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FileUtils;
import org.springframework.scheduling.annotation.Async;
import org.springframework.util.Assert;

import javax.annotation.PostConstruct;
import javax.annotation.PreDestroy;
import java.io.File;
import java.io.IOException;
import java.io.InputStream;
import java.nio.charset.Charset;
import java.util.Collection;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import java.util.stream.Stream;

import static java.util.Arrays.asList;

@Slf4j
@RequiredArgsConstructor
public class SubtitleServiceImpl implements SubtitleService {
    public static final String SUBTITLE_PROPERTY = "activeSubtitle";

    private final ObjectProperty<Subtitle> activeSubtitle = new SimpleObjectProperty<>(this, SUBTITLE_PROPERTY, null);
    private final Collection<Parser> parsers = asList(new SrtParser(), new VttParser());
    private final SettingsService settingsService;
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
        return parsers.stream()
                .filter(e -> e.support(type))
                .findFirst()
                .map(e -> e.parse(subtitle.getCues()))
                .orElseThrow(() -> new SubtitleParsingException("No parser found for type " + type));
    }

    @Override
    public SubtitleInfo getDefaultOrInterfaceLanguage(List<SubtitleInfo> subtitles) {
        Assert.notNull(subtitles, "subtitles cannot be null");
        return delegate.getDefaultOrInterfaceLanguage(subtitles);
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        initializeListeners();
    }

    private void initializeListeners() {
        SubtitleSettings settings = getSubtitleSettings();

        settings.addListener(evt -> {
            if (SubtitleSettings.DIRECTORY_PROPERTY.equals(evt.getPropertyName())) {
                // clean old directory
                if (settings.isAutoCleaningEnabled())
                    cleanCacheDirectory((File) evt.getOldValue());
            }
        });
    }

    //endregion

    //region PreDestroy

    @PreDestroy
    private void destroy() {
        var settings = getSubtitleSettings();

        if (settings.isAutoCleaningEnabled() && settings.getDirectory().exists()) {
            cleanCacheDirectory(settings.getDirectory());
        }
    }

    //endregion

    //region Functions

    private SubtitleSettings getSubtitleSettings() {
        return getSettings().getSubtitleSettings();
    }

    private ApplicationSettings getSettings() {
        return settingsService.getSettings();
    }

    private void cleanCacheDirectory(File directory) {
        try {
            log.info("Cleaning subtitles directory {}", directory);
            FileUtils.cleanDirectory(directory);
        } catch (IOException ex) {
            log.error(ex.getMessage(), ex);
        }
    }

    //endregion
}