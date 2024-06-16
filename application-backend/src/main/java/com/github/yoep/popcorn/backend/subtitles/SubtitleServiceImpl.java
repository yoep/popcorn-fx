package com.github.yoep.popcorn.backend.subtitles;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.media.providers.Episode;
import com.github.yoep.popcorn.backend.media.providers.MovieDetails;
import com.github.yoep.popcorn.backend.media.providers.ShowDetails;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import com.github.yoep.popcorn.backend.subtitles.ffi.SubtitleEvent;
import com.github.yoep.popcorn.backend.subtitles.ffi.SubtitleEventCallback;
import com.github.yoep.popcorn.backend.subtitles.ffi.SubtitleInfoSet;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleMatcher;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitlePreference;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitlePreferenceTag;
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

import static java.util.Arrays.asList;

@Slf4j
public class SubtitleServiceImpl implements SubtitleService, SubtitleEventCallback {
    private final FxLib fxLib;
    private final PopcornFx instance;
    private final ExecutorService executorService;

    private final ConcurrentLinkedDeque<com.github.yoep.popcorn.backend.subtitles.model.SubtitleEventCallback> listeners = new ConcurrentLinkedDeque<>();

    public SubtitleServiceImpl(FxLib fxLib, PopcornFx instance, ExecutorService executorService) {
        this.fxLib = fxLib;
        this.instance = instance;
        this.executorService = executorService;
        init();
    }

    //region Properties

    @Override
    public boolean isDisabled() {
        try (var preference = fxLib.retrieve_subtitle_preference(instance)) {
            return preference.getTag() == SubtitlePreferenceTag.DISABLED;
        }
    }

    @Override
    public SubtitleInfo none() {
        var info = fxLib.subtitle_none();
        try (info) {
            return SubtitleInfo.from(info);
        } finally {
            fxLib.dispose_subtitle_info(info);
        }
    }

    @Override
    public SubtitleInfo custom() {
        var info = fxLib.subtitle_custom();
        try (info) {
            return SubtitleInfo.from(info);
        } finally {
            fxLib.dispose_subtitle_info(info);
        }
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
                    Stream.concat(defaultOptions().stream(), subtitles.stream().map(SubtitleInfo::from))
                            .toList(), executorService);
        }
    }

    @Override
    public CompletableFuture<List<SubtitleInfo>> retrieveSubtitles(final ShowDetails media, final Episode episode) {
        Objects.requireNonNull(media, "media cannot be null");
        Objects.requireNonNull(episode, "episode cannot be null");
        try (var subtitle_set = fxLib.episode_subtitles(instance, media, episode)) {
            var subtitles = Optional.ofNullable(subtitle_set)
                    .map(SubtitleInfoSet::getSubtitles)
                    .orElse(Collections.emptyList());

            log.debug("Retrieved episode subtitle {}", subtitles);
            return CompletableFuture.supplyAsync(() ->
                    Stream.concat(defaultOptions().stream(), subtitles.stream().map(SubtitleInfo::from))
                            .toList(), executorService);
        }
    }

    @Override
    public CompletableFuture<List<SubtitleInfo>> retrieveSubtitles(final String filename) {
        Objects.requireNonNull(filename, "filename cannot be null");
        try (var subtitle_set = fxLib.filename_subtitles(instance, filename)) {
            var subtitles = Optional.ofNullable(subtitle_set)
                    .map(SubtitleInfoSet::getSubtitles)
                    .orElse(Collections.emptyList());

            return CompletableFuture.supplyAsync(() ->
                    Stream.concat(defaultOptions().stream(), subtitles.stream().map(SubtitleInfo::from))
                            .toList(), executorService);
        }
    }

    @Override
    public CompletableFuture<Subtitle> downloadAndParse(SubtitleInfo subtitleInfo, SubtitleMatcher.ByReference matcher) {
        Objects.requireNonNull(subtitleInfo, "subtitleInfo cannot be null");
        Objects.requireNonNull(matcher, "matcher cannot be null");

        try (var info = com.github.yoep.popcorn.backend.subtitles.ffi.SubtitleInfo.ByReference.from(subtitleInfo)) {
            return CompletableFuture.supplyAsync(() -> {
                log.debug("Starting subtitle download subtitleInfo: {}, matcher: {}", subtitleInfo, matcher);
                var subtitle = fxLib.download_and_parse_subtitle(instance, info, matcher);
                log.info("Downloaded and parsed subtitle info {} to {}", subtitleInfo, subtitle.getFilepath());
                return subtitle;
            }, executorService);
        }
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

        var ffiSubtitles = subtitles.stream()
                .map(com.github.yoep.popcorn.backend.subtitles.ffi.SubtitleInfo.ByReference::from)
                .toArray(com.github.yoep.popcorn.backend.subtitles.ffi.SubtitleInfo[]::new);
        try (var set = new SubtitleInfoSet.ByReference(asList(ffiSubtitles))) {
            try (var subtitle = fxLib.select_or_default_subtitle(instance, set)) {
                return SubtitleInfo.from(subtitle);
            }
        }
    }

    @Override
    public SubtitlePreference preference() {
        log.trace("Retrieving subtitle preference");
        var preference = fxLib.retrieve_subtitle_preference(instance);

        try (preference) {
            return SubtitlePreference.from(preference);
        } finally {
            fxLib.dispose_subtitle_preference(preference);
        }
    }

    @Override
    public void updateSubtitle(SubtitleInfo subtitle) {
        if (subtitle != null) {
            try (var preference = new com.github.yoep.popcorn.backend.subtitles.ffi.SubtitlePreference.ByReference(subtitle.language())) {
                log.trace("Updating subtitle to {}", subtitle);
                fxLib.update_subtitle_preference(instance, preference);
            }
        } else {
            log.trace("Clearing the preferred subtitle");
            fxLib.reset_subtitle(instance);
        }
    }

    @Override
    public void updatePreferredLanguage(SubtitleLanguage language) {
        log.trace("Updating preferred subtitle language to {}", language);
        try (var preference = new com.github.yoep.popcorn.backend.subtitles.ffi.SubtitlePreference.ByReference(language)) {
            fxLib.update_subtitle_preference(instance, preference);
        }
    }

    @Override
    public void register(com.github.yoep.popcorn.backend.subtitles.model.SubtitleEventCallback callback) {
        Objects.requireNonNull(callback, "callback cannot be null");
        listeners.add(callback);
    }

    @Override
    public void disableSubtitle() {
        log.trace("Disabling subtitle");
        fxLib.update_subtitle_preference(instance, com.github.yoep.popcorn.backend.subtitles.ffi.SubtitlePreference.ByReference.disabled());
    }

    @Override
    public void reset() {
        log.trace("Resetting the subtitle selection");
        fxLib.reset_subtitle(instance);
    }

    @Override
    public void cleanup() {
        fxLib.cleanup_subtitles_directory(instance);
    }

    @Override
    public void callback(SubtitleEvent.ByValue event) {
        com.github.yoep.popcorn.backend.subtitles.model.SubtitleEvent modelEvent;

        try (event) {
            log.debug("Received subtitle event callback {}", event);
            modelEvent = com.github.yoep.popcorn.backend.subtitles.model.SubtitleEvent.from(event);
        }

        for (var listener : listeners) {
            try {
                listener.callback(modelEvent);
            } catch (Exception ex) {
                log.error("Failed to invoke subtitle callback, {}", ex.getMessage(), ex);
            }
        }
    }

    //endregion

    private void init() {
        fxLib.register_subtitle_callback(instance, this);
    }

    private List<SubtitleInfo> defaultOptions() {
        try (var set = fxLib.default_subtitle_options(instance)) {
            return set.getSubtitles().stream()
                    .map(SubtitleInfo::from)
                    .toList();
        }
    }
}
