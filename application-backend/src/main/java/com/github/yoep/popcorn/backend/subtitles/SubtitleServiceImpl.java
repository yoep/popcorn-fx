package com.github.yoep.popcorn.backend.subtitles;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.backend.media.providers.models.MovieDetails;
import com.github.yoep.popcorn.backend.media.providers.models.ShowDetails;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfoSet;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleMatcher;
import lombok.extern.slf4j.Slf4j;

import java.util.Collections;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ConcurrentLinkedDeque;
import java.util.concurrent.ExecutorService;
import java.util.stream.Collectors;
import java.util.stream.Stream;

@Slf4j
public class SubtitleServiceImpl implements SubtitleService {
    private final FxLib fxLib;
    private final PopcornFx instance;
    private final ExecutorService executorService;

    private final SubtitleEventCallback callback = createCallback();
    private final ConcurrentLinkedDeque<SubtitleEventCallback> listeners = new ConcurrentLinkedDeque<>();
    private final Object mutex = new Object();

    public SubtitleServiceImpl(FxLib fxLib, PopcornFx instance, ExecutorService executorService) {
        this.fxLib = fxLib;
        this.instance = instance;
        this.executorService = executorService;
        init();
    }

    //region Properties

    @Override
    public boolean isDisabled() {
        return fxLib.is_subtitle_disabled(instance) == 1;
    }

    @Override
    public SubtitleInfo none() {
        return fxLib.subtitle_none();
    }

    @Override
    public SubtitleInfo custom() {
        return fxLib.subtitle_custom();
    }

    //endregion

    //region Methods

    @Override
    public CompletableFuture<List<SubtitleInfo>> retrieveSubtitles(final MovieDetails media) {
        Objects.requireNonNull(media, "media cannot be null");
        try (var set = fxLib.movie_subtitles(instance, media)) {
            var subtitles = Optional.ofNullable(set)
                    .map(SubtitleInfoSet::getSubtitles)
                    .orElse(Collections.emptyList());

            log.debug("Retrieved movie subtitles {}", subtitles);
            return CompletableFuture.supplyAsync(() ->
                    Stream.concat(defaultOptions().stream(), subtitles.stream()).toList(), executorService);
        }
    }

    @Override
    public CompletableFuture<List<SubtitleInfo>> retrieveSubtitles(final ShowDetails media, final Episode episode) {
        Objects.requireNonNull(media, "media cannot be null");
        Objects.requireNonNull(episode, "episode cannot be null");
        var subtitles = Optional.ofNullable(fxLib.episode_subtitles(instance, media, episode))
                .map(SubtitleInfoSet::getSubtitles)
                .orElse(Collections.emptyList());

        log.debug("Retrieved episode subtitle {}", subtitles);
        return CompletableFuture.supplyAsync(() ->
                Stream.concat(defaultOptions().stream(), subtitles.stream()).toList(), executorService);
    }

    @Override
    public CompletableFuture<List<SubtitleInfo>> retrieveSubtitles(final String filename) {
        Objects.requireNonNull(filename, "filename cannot be null");
        var subtitles = Optional.ofNullable(fxLib.filename_subtitles(instance, filename))
                .map(SubtitleInfoSet::getSubtitles)
                .orElse(Collections.emptyList());

        return CompletableFuture.supplyAsync(() ->
                Stream.concat(defaultOptions().stream(), subtitles.stream()).toList(), executorService);
    }

    @Override
    public CompletableFuture<Subtitle> downloadAndParse(SubtitleInfo subtitleInfo, SubtitleMatcher.ByValue matcher) {
        Objects.requireNonNull(subtitleInfo, "subtitleInfo cannot be null");
        Objects.requireNonNull(matcher, "matcher cannot be null");

        return CompletableFuture.supplyAsync(() -> {
            synchronized (mutex) {
                log.debug("Starting subtitle download subtitleInfo: {}, matcher: {}", subtitleInfo, matcher);
                var subtitle = fxLib.download_and_parse_subtitle(instance, subtitleInfo, matcher);
                log.info("Downloaded and parsed subtitle info {} to {}", subtitleInfo, subtitle.getFilepath());
                return subtitle;
            }
        }, executorService);
    }

    @Override
    public SubtitleInfo getDefaultOrInterfaceLanguage(List<SubtitleInfo> subtitles) {
        Objects.requireNonNull(subtitles, "subtitles cannot be null");
        subtitles = subtitles.stream()
                .filter(e -> !e.isSpecial())
                .collect(Collectors.toList());

        if (subtitles.isEmpty()) {
            return none();
        }

        try (var set = new SubtitleInfoSet.ByReference(subtitles)) {
            synchronized (mutex) {
                return fxLib.select_or_default_subtitle(instance, set);
            }
        }
    }

    @Override
    public Optional<SubtitleInfo.ByReference> preferredSubtitle() {
        return Optional.ofNullable(fxLib.retrieve_preferred_subtitle(instance));
    }

    @Override
    public SubtitleLanguage preferredSubtitleLanguage() {
        synchronized (mutex) {
            return fxLib.retrieve_preferred_subtitle_language(instance);
        }
    }

    @Override
    public void updateSubtitle(SubtitleInfo subtitle) {
        synchronized (mutex) {
            if (subtitle != null) {
                log.trace("Updating subtitle to {}", subtitle);
                fxLib.update_subtitle(instance, subtitle);
            } else {
                log.trace("Clearing the preferred subtitle");
                fxLib.reset_subtitle(instance);
            }
        }
    }

    @Override
    public void updateCustomSubtitle(String subtitleFilepath) {
        Objects.requireNonNull(subtitleFilepath, "subtitleFilepath cannot be null");
        synchronized (mutex) {
            log.trace("Updating subtitle custom filepath to {}", subtitleFilepath);
            fxLib.update_subtitle_custom_file(instance, subtitleFilepath);
        }
    }

    @Override
    public void register(SubtitleEventCallback callback) {
        Objects.requireNonNull(callback, "callback cannot be null");
        listeners.add(callback);
    }

    @Override
    public void disableSubtitle() {
        synchronized (mutex) {
            log.trace("Disabling subtitle");
            fxLib.disable_subtitle(instance);
        }
    }

    @Override
    public void reset() {
        synchronized (mutex) {
            log.trace("Resetting the subtitle selection");
            fxLib.reset_subtitle(instance);
        }
    }

    @Override
    public void cleanup() {
        fxLib.cleanup_subtitles_directory(instance);
    }

    //endregion

    private void init() {
        fxLib.register_subtitle_callback(instance, callback);
    }

    private List<SubtitleInfo> defaultOptions() {
        try (var set = fxLib.default_subtitle_options(instance)) {
            return set.getSubtitles();
        }
    }

    private SubtitleEventCallback createCallback() {
        return event -> {
            try (event) {
                log.debug("Received subtitle event callback {}", event);
                for (var listener : listeners) {
                    try {
                        listener.callback(event);
                    } catch (Exception ex) {
                        log.error("Failed to invoke subtitle callback, {}", ex.getMessage(), ex);
                    }
                }
            }
        };
    }
}
