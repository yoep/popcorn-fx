package com.github.yoep.popcorn.backend.subtitles;

import com.github.yoep.popcorn.backend.lib.FxCallback;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.FxChannelException;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.*;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle.Info;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle.Language;
import com.github.yoep.popcorn.backend.media.Episode;
import com.github.yoep.popcorn.backend.media.MediaHelper;
import com.github.yoep.popcorn.backend.media.MovieDetails;
import com.github.yoep.popcorn.backend.media.ShowDetails;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleMatcher;
import lombok.extern.slf4j.Slf4j;

import java.util.List;
import java.util.Objects;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ConcurrentLinkedDeque;

@Slf4j
public class SubtitleServiceImpl implements ISubtitleService, FxCallback<SubtitleEvent> {
    private final FxChannel fxChannel;

    private final ConcurrentLinkedDeque<FxCallback<SubtitleEvent>> listeners = new ConcurrentLinkedDeque<>();

    public SubtitleServiceImpl(FxChannel fxChannel) {
        Objects.requireNonNull(fxChannel, "fxChannel cannot be null");
        this.fxChannel = fxChannel;
        init();
    }

    @Override
    public CompletableFuture<List<ISubtitleInfo>> defaultSubtitles() {
        return fxChannel.send(GetDefaultSubtitlesRequest.getDefaultInstance(), GetDefaultSubtitlesResponse.parser())
                .thenApply(GetDefaultSubtitlesResponse::getSubtitlesList)
                .thenApply(subtitles -> subtitles.stream()
                        .map(SubtitleInfoWrapper::new)
                        .map(info -> (ISubtitleInfo) info)
                        .toList());
    }

    @Override
    public CompletableFuture<List<ISubtitleInfo>> retrieveSubtitles(final MovieDetails media) {
        Objects.requireNonNull(media, "media cannot be null");
        return fxChannel.send(GetMediaAvailableSubtitlesRequest.newBuilder()
                        .setItem(MediaHelper.getItem(media))
                        .build(), GetMediaAvailableSubtitlesResponse.parser())
                .thenApply(this::mapAvailableSubtitlesResponse)
                .thenApply(subtitles -> subtitles.stream()
                        .map(SubtitleInfoWrapper::new)
                        .map(info -> (ISubtitleInfo) info)
                        .toList());
    }

    @Override
    public CompletableFuture<List<ISubtitleInfo>> retrieveSubtitles(final ShowDetails media, final Episode episode) {
        Objects.requireNonNull(media, "media cannot be null");
        Objects.requireNonNull(episode, "episode cannot be null");
        return fxChannel.send(GetMediaAvailableSubtitlesRequest.newBuilder()
                        .setItem(MediaHelper.getItem(media))
                        .setSubItem(MediaHelper.getItem(episode))
                        .build(), GetMediaAvailableSubtitlesResponse.parser())
                .thenApply(this::mapAvailableSubtitlesResponse)
                .thenApply(subtitles -> subtitles.stream()
                        .map(SubtitleInfoWrapper::new)
                        .map(info -> (ISubtitleInfo) info)
                        .toList());
    }

    @Override
    public CompletableFuture<List<ISubtitleInfo>> retrieveSubtitles(final String filename) {
        Objects.requireNonNull(filename, "filename cannot be null");
        return fxChannel.send(GetFileAvailableSubtitlesRequest.newBuilder()
                        .setFilename(filename)
                        .build(), GetFileAvailableSubtitlesResponse.parser())
                .thenApply(response -> {
                    if (response.getResult() == Response.Result.OK) {
                        return response.getSubtitlesList();
                    } else {
                        log.error("Failed to retrieve subtitles for {}, {}", filename, response.getError());
                        throw new FxChannelException("Failed to retrieve subtitles");
                    }
                })
                .thenApply(subtitles -> subtitles.stream()
                        .map(SubtitleInfoWrapper::new)
                        .map(info -> (ISubtitleInfo) info)
                        .toList());
    }

    @Override
    public CompletableFuture<ISubtitle> downloadAndParse(Info subtitleInfo, SubtitleMatcher.ByReference matcher) {
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
    public CompletableFuture<ISubtitleInfo> getDefaultOrInterfaceLanguage(List<ISubtitleInfo> subtitles) {
        Objects.requireNonNull(subtitles, "subtitles cannot be null");
        subtitles = subtitles.stream()
                .filter(e -> !(e.isNone() || e.isCustom()))
                .toList();

        if (subtitles.isEmpty()) {
            return defaultSubtitles().thenApply(List::getFirst);
        }

        return fxChannel.send(GetPreferredSubtitleRequest.newBuilder()
                        .addAllSubtitles(subtitles.stream()
                                .filter(e -> e instanceof SubtitleInfoWrapper)
                                .map(e -> (SubtitleInfoWrapper) e)
                                .map(SubtitleInfoWrapper::proto)
                                .toList())
                        .build(), GetPreferredSubtitleResponse.parser())
                .thenApply(GetPreferredSubtitleResponse::getSubtitle)
                .thenApply(SubtitleInfoWrapper::new);
    }

    @Override
    public CompletableFuture<SubtitlePreference> preference() {
        log.trace("Retrieving subtitle preference");
        return fxChannel.send(GetSubtitlePreferenceRequest.getDefaultInstance(), GetSubtitlePreferenceResponse.parser())
                .thenApply(GetSubtitlePreferenceResponse::getPreference);
    }

    @Override
    public void updatePreferredLanguage(Language language) {
        log.trace("Updating preferred subtitle language to {}", language);
        fxChannel.send(UpdateSubtitlePreferenceRequest.newBuilder()
                .setPreference(SubtitlePreference.newBuilder()
                        .setPreference(SubtitlePreference.Preference.LANGUAGE)
                        .setLanguage(language)
                        .build())
                .build());
    }

    @Override
    public void register(FxCallback<SubtitleEvent> callback) {
        Objects.requireNonNull(callback, "callback cannot be null");
        listeners.add(callback);
    }

    @Override
    public void disableSubtitle() {
        log.trace("Disabling subtitle");
        fxChannel.send(UpdateSubtitlePreferenceRequest.newBuilder()
                .setPreference(SubtitlePreference.newBuilder()
                        .setPreference(SubtitlePreference.Preference.DISABLED)
                        .build())
                .build());
    }

    @Override
    public void reset() {
        log.trace("Resetting the subtitle selection");
        fxChannel.send(ResetSubtitleRequest.getDefaultInstance());
    }

    @Override
    public void cleanup() {
        fxChannel.send(CleanSubtitlesDirectoryRequest.getDefaultInstance());
    }

    @Override
    public void callback(SubtitleEvent message) {
        try {
            listeners.forEach(e -> e.callback(message));
        } catch (Exception e) {
            log.error("Failed to process subtitle event", e);
        }
    }

    private List<Info> mapAvailableSubtitlesResponse(GetMediaAvailableSubtitlesResponse response) {
        if (response.getResult() == Response.Result.OK) {
            return response.getSubtitlesList();
        } else {
            log.error("Failed to retrieve available subtitles, {}", response.getError());
            throw new FxChannelException("Failed to retrieve subtitles");
        }
    }

    private void init() {
        fxChannel.subscribe(FxChannel.typeFrom(SubtitleEvent.class), SubtitleEvent.parser(), this);
    }
}
