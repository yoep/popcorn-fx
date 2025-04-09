package com.github.yoep.popcorn.backend.subtitles;

import com.github.yoep.popcorn.backend.lib.FxCallback;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.*;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle.Info;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle.Language;
import com.github.yoep.popcorn.backend.media.Episode;
import com.github.yoep.popcorn.backend.media.MovieDetails;
import com.github.yoep.popcorn.backend.media.ShowDetails;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleMatcher;
import lombok.extern.slf4j.Slf4j;

import java.util.List;
import java.util.Objects;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ConcurrentLinkedDeque;

@Slf4j
public class SubtitleServiceImpl implements SubtitleService, FxCallback<SubtitleEvent> {
    private final FxChannel fxChannel;

    private final ConcurrentLinkedDeque<FxCallback<SubtitleEvent>> listeners = new ConcurrentLinkedDeque<>();

    public SubtitleServiceImpl(FxChannel fxChannel) {
        Objects.requireNonNull(fxChannel, "fxChannel cannot be null");
        this.fxChannel = fxChannel;
        init();
    }

    //region Properties

    @Override
    public CompletableFuture<Boolean> isDisabled() {
        return fxChannel.send(GetSubtitlePreferenceRequest.getDefaultInstance(), GetSubtitlePreferenceResponse.parser())
                .thenApply(GetSubtitlePreferenceResponse::getPreference)
                .thenApply(e -> e.getLanguage() == Language.NONE);
    }

    @Override
    public CompletableFuture<Info> none() {
        return fxChannel.send(GetSubtitleNoneRequest.getDefaultInstance(), GetSubtitleNoneResponse.parser())
                .thenApply(GetSubtitleNoneResponse::getInfo);
    }

    @Override
    public CompletableFuture<Info> custom() {
        return fxChannel.send(GetSubtitleCustomRequest.getDefaultInstance(), GetSubtitleCustomResponse.parser())
                .thenApply(GetSubtitleCustomResponse::getInfo);
    }

    //endregion

    //region Methods

    @Override
    public CompletableFuture<List<Info>> retrieveSubtitles(final MovieDetails media) {
        Objects.requireNonNull(media, "media cannot be null");
//        try (var set = fxLib.movie_subtitles(instance, media)) {
//            var subtitles = Optional.ofNullable(set)
//                    .map(SubtitleInfoSet::getSubtitles)
//                    .orElse(Collections.emptyList());
//
//            log.debug("Retrieved movie subtitles {}", subtitles);
//            return CompletableFuture.supplyAsync(() ->
//                    Stream.concat(defaultOptions().stream(), subtitles.stream().map(SubtitleInfo::from))
//                            .toList(), executorService);
//        }
        return null;
    }

    @Override
    public CompletableFuture<List<Info>> retrieveSubtitles(final ShowDetails media, final Episode episode) {
        Objects.requireNonNull(media, "media cannot be null");
        Objects.requireNonNull(episode, "episode cannot be null");
//        try (var subtitle_set = fxLib.episode_subtitles(instance, media, episode)) {
//            var subtitles = Optional.ofNullable(subtitle_set)
//                    .map(SubtitleInfoSet::getSubtitles)
//                    .orElse(Collections.emptyList());
//
//            log.debug("Retrieved episode subtitle {}", subtitles);
//            return CompletableFuture.supplyAsync(() ->
//                    Stream.concat(defaultOptions().stream(), subtitles.stream().map(SubtitleInfo::from))
//                            .toList(), executorService);
//        }
        return null;
    }

    @Override
    public CompletableFuture<List<Info>> retrieveSubtitles(final String filename) {
        Objects.requireNonNull(filename, "filename cannot be null");
//        try (var subtitle_set = fxLib.filename_subtitles(instance, filename)) {
//            var subtitles = Optional.ofNullable(subtitle_set)
//                    .map(SubtitleInfoSet::getSubtitles)
//                    .orElse(Collections.emptyList());
//
//            return CompletableFuture.supplyAsync(() ->
//                    Stream.concat(defaultOptions().stream(), subtitles.stream().map(SubtitleInfo::from))
//                            .toList(), executorService);
//        }
        return null;
    }

    @Override
    public CompletableFuture<Subtitle> downloadAndParse(Info subtitleInfo, SubtitleMatcher.ByReference matcher) {
        Objects.requireNonNull(subtitleInfo, "subtitleInfo cannot be null");
        Objects.requireNonNull(matcher, "matcher cannot be null");

//        try (var info = com.github.yoep.popcorn.backend.subtitles.ffi.SubtitleInfo.ByReference.from(subtitleInfo)) {
//            return CompletableFuture.supplyAsync(() -> {
//                log.debug("Starting subtitle download subtitleInfo: {}, matcher: {}", subtitleInfo, matcher);
//                var subtitle = fxLib.download_and_parse_subtitle(instance, info, matcher);
//                log.info("Downloaded and parsed subtitle info {} to {}", subtitleInfo, subtitle.getFilepath());
//                return subtitle;
//            }, executorService);
//        }
        return null;
    }

    @Override
    public Info getDefaultOrInterfaceLanguage(List<Info> subtitles) {
        Objects.requireNonNull(subtitles, "subtitles cannot be null");
//        subtitles = subtitles.stream()
//                .filter(e -> !e.isSpecial())
//                .collect(Collectors.toList());

//        if (subtitles.isEmpty()) {
//            return none();
//        }
//
//        var ffiSubtitles = subtitles.stream()
//                .map(com.github.yoep.popcorn.backend.subtitles.ffi.SubtitleInfo.ByReference::from)
//                .toArray(com.github.yoep.popcorn.backend.subtitles.ffi.SubtitleInfo[]::new);
//        try (var set = new SubtitleInfoSet.ByReference(asList(ffiSubtitles))) {
//            try (var subtitle = fxLib.select_or_default_subtitle(instance, set)) {
//                return SubtitleInfo.from(subtitle);
//            }
//        }
        return null;
    }

    @Override
    public CompletableFuture<SubtitlePreference> preference() {
        log.trace("Retrieving subtitle preference");
        return fxChannel.send(GetSubtitlePreferenceRequest.getDefaultInstance(), GetSubtitlePreferenceResponse.parser())
                .thenApply(GetSubtitlePreferenceResponse::getPreference);
    }

    @Override
    public void updateSubtitle(Info subtitle) {
//        if (subtitle != null) {
//            try (var preference = new com.github.yoep.popcorn.backend.subtitles.ffi.SubtitlePreference.ByReference(subtitle.language())) {
//                log.trace("Updating subtitle to {}", subtitle);
//                fxLib.update_subtitle_preference(instance, preference);
//            }
//        } else {
//            log.trace("Clearing the preferred subtitle");
//            fxLib.reset_subtitle(instance);
//        }
    }

    @Override
    public void updatePreferredLanguage(Language language) {
        log.trace("Updating preferred subtitle language to {}", language);

    }

    @Override
    public void register(FxCallback<SubtitleEvent> callback) {
        Objects.requireNonNull(callback, "callback cannot be null");
        listeners.add(callback);
    }

    @Override
    public void disableSubtitle() {
        log.trace("Disabling subtitle");
//        fxLib.update_subtitle_preference(instance, com.github.yoep.popcorn.backend.subtitles.ffi.SubtitlePreference.ByReference.disabled());
    }

    @Override
    public void reset() {
        log.trace("Resetting the subtitle selection");
//        fxLib.reset_subtitle(instance);
    }

    @Override
    public void cleanup() {
        // TODO
    }

    //endregion

    @Override
    public void callback(SubtitleEvent message) {
        try {
            listeners.forEach(e -> e.callback(message));
        } catch (Exception e) {
            log.error("Failed to process subtitle event", e);
        }
    }

    private void init() {
        fxChannel.subscribe(FxChannel.typeFrom(SubtitleEvent.class), SubtitleEvent.parser(), this);
    }
}
