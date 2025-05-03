package com.github.yoep.popcorn.backend.subtitles;

import com.github.yoep.popcorn.backend.lib.FxCallback;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.*;
import com.github.yoep.popcorn.backend.media.Episode;
import com.github.yoep.popcorn.backend.media.MovieDetails;
import com.github.yoep.popcorn.backend.media.ShowDetails;
import com.google.protobuf.Parser;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicReference;

import static java.util.Arrays.asList;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class SubtitleServiceImplTest {
    @Mock
    private FxChannel fxChannel;
    private SubtitleServiceImpl service;

    private final AtomicReference<FxCallback<SubtitleEvent>> eventCallbackHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        doAnswer(invocations -> {
            eventCallbackHolder.set((FxCallback<SubtitleEvent>) invocations.getArgument(2, FxCallback.class));
            return null;
        }).when(fxChannel).subscribe(eq(FxChannel.typeFrom(SubtitleEvent.class)), isA(Parser.class), isA(FxCallback.class));

        service = new SubtitleServiceImpl(fxChannel);
    }

    @Test
    void testDefaultSubtitles() {
        var defaultSubtitles = asList(
                Subtitle.Info.newBuilder()
                        .setLanguage(Subtitle.Language.NONE)
                        .build(),
                Subtitle.Info.newBuilder()
                        .setLanguage(Subtitle.Language.CUSTOM)
                        .build()
        );
        var expectedResult = asList(new SubtitleInfoWrapper(defaultSubtitles.get(0)), new SubtitleInfoWrapper(defaultSubtitles.get(1)));
        when(fxChannel.send(isA(GetDefaultSubtitlesRequest.class), isA(Parser.class))).thenReturn(CompletableFuture.completedFuture(GetDefaultSubtitlesResponse.newBuilder()
                .addAllSubtitles(defaultSubtitles)
                .build()));

        var result = service.defaultSubtitles().resultNow();

        verify(fxChannel).send(isA(GetDefaultSubtitlesRequest.class), isA(Parser.class));
        assertEquals(expectedResult, result);
    }

    @Test
    void testRetrieveSubtitles_movieDetails() {
        var media = new MovieDetails(Media.MovieDetails.newBuilder()
                .setImdbId("tt1222000")
                .build());
        var request = new AtomicReference<GetMediaAvailableSubtitlesRequest>();
        var subtitle = new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                .setLanguage(Subtitle.Language.ENGLISH)
                .build());
        when(fxChannel.send(isA(GetMediaAvailableSubtitlesRequest.class), isA(Parser.class))).thenAnswer(invocations -> {
            request.set(invocations.getArgument(0, GetMediaAvailableSubtitlesRequest.class));
            return CompletableFuture.completedFuture(GetMediaAvailableSubtitlesResponse.newBuilder()
                    .addSubtitles(subtitle.proto())
                    .build());
        });

        var result = service.retrieveSubtitles(media).resultNow();

        verify(fxChannel).send(isA(GetMediaAvailableSubtitlesRequest.class), isA(Parser.class));
        assertEquals(media.proto(), request.get().getItem().getMovieDetails());
        assertEquals(1, result.size(), "expected a subtitle to have been returned");
        assertEquals(subtitle, result.getFirst());
    }

    @Test
    void testRetrieveSubtitles_ShowEpisode() {
        var episode = new Episode(Media.Episode.newBuilder()
                .setTvdbId("tt3300011")
                .setEpisode(1)
                .setSeason(1)
                .build());
        var media = new ShowDetails(Media.ShowDetails.newBuilder()
                .setImdbId("tt1222000")
                .addEpisodes(episode.proto())
                .build());
        var request = new AtomicReference<GetMediaAvailableSubtitlesRequest>();
        var subtitle = new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                .setLanguage(Subtitle.Language.DANISH)
                .build());
        when(fxChannel.send(isA(GetMediaAvailableSubtitlesRequest.class), isA(Parser.class))).thenAnswer(invocations -> {
            request.set(invocations.getArgument(0, GetMediaAvailableSubtitlesRequest.class));
            return CompletableFuture.completedFuture(GetMediaAvailableSubtitlesResponse.newBuilder()
                    .addSubtitles(subtitle.proto())
                    .build());
        });

        var result = service.retrieveSubtitles(media, episode).resultNow();

        verify(fxChannel).send(isA(GetMediaAvailableSubtitlesRequest.class), isA(Parser.class));
        assertEquals(media.proto(), request.get().getItem().getShowDetails());
        assertEquals(episode.proto(), request.get().getSubItem().getEpisode());
        assertEquals(1, result.size(), "expected a subtitle to have been returned");
        assertEquals(subtitle, result.getFirst());
    }

    @Test
    void testRetrieveSubtitles_filename() {
        var filename = "my-video.mp4";
        var request = new AtomicReference<GetFileAvailableSubtitlesRequest>();
        var subtitle = new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                .setLanguage(Subtitle.Language.FRENCH)
                .build());
        when(fxChannel.send(isA(GetFileAvailableSubtitlesRequest.class), isA(Parser.class))).thenAnswer(invocations -> {
            request.set(invocations.getArgument(0, GetFileAvailableSubtitlesRequest.class));
            return CompletableFuture.completedFuture(GetFileAvailableSubtitlesResponse.newBuilder()
                    .addSubtitles(subtitle.proto())
                    .build());
        });

        var result = service.retrieveSubtitles(filename).resultNow();

        verify(fxChannel).send(isA(GetFileAvailableSubtitlesRequest.class), isA(Parser.class));
        assertEquals(filename, request.get().getFilename());
        assertEquals(1, result.size(), "expected a subtitle to have been returned");
        assertEquals(subtitle, result.getFirst());
    }

    @Test
    void testDownloadAndParse() {
        var subtitleInfo = new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                .setImdbId("tt1234444")
                .setLanguage(Subtitle.Language.FINNISH)
                .build());
        var matcher = Subtitle.Matcher.newBuilder()
                .setFilename("MyFilename.mp4")
                .setQuality("720p")
                .build();
        var subtitle = new SubtitleWrapper(Subtitle.newBuilder()
                .setInfo(subtitleInfo.proto())
                .build());
        var request = new AtomicReference<DownloadAndParseSubtitleRequest>();
        when(fxChannel.send(isA(DownloadAndParseSubtitleRequest.class), isA(Parser.class))).thenAnswer(invocations -> {
            request.set(invocations.getArgument(0, DownloadAndParseSubtitleRequest.class));
            return CompletableFuture.completedFuture(DownloadAndParseSubtitleResponse.newBuilder()
                    .setResult(Response.Result.OK)
                    .setSubtitle(subtitle.proto())
                    .build());
        });

        var result = service.downloadAndParse(subtitleInfo, matcher).resultNow();

        verify(fxChannel).send(isA(DownloadAndParseSubtitleRequest.class), isA(Parser.class));
        assertEquals(subtitleInfo.proto(), request.get().getInfo());
        assertEquals(matcher, request.get().getMatcher());
        assertEquals(subtitle, result);
    }

    @Test
    void testGetDefaultOrInterfaceLanguage() {
        var subtitleInfo = new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                .setImdbId("ImdbId1")
                .setLanguage(Subtitle.Language.ARABIC)
                .build());
        var request = new AtomicReference<GetPreferredSubtitleRequest>();
        when(fxChannel.send(isA(GetPreferredSubtitleRequest.class), isA(Parser.class))).thenAnswer(invocations -> {
            request.set(invocations.getArgument(0, GetPreferredSubtitleRequest.class));
            return CompletableFuture.completedFuture(GetPreferredSubtitleResponse.newBuilder()
                    .setSubtitle(subtitleInfo.proto())
                    .build());
        });

        var result = service.getDefaultOrInterfaceLanguage(asList(subtitleInfo, new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                        .setImdbId("ImdbId2")
                        .setLanguage(Subtitle.Language.BASQUE)
                        .build())))
                .resultNow();

        verify(fxChannel).send(isA(GetPreferredSubtitleRequest.class), isA(Parser.class));
        assertEquals(subtitleInfo.proto(), request.get().getSubtitles(0));
        assertEquals(subtitleInfo, result);
    }

    @Test
    void testDisableSubtitle() {
        var request = new AtomicReference<UpdateSubtitlePreferenceRequest>();
        doAnswer(invocations -> {
            request.set(invocations.getArgument(0, UpdateSubtitlePreferenceRequest.class));
            return null;
        }).when(fxChannel).send(isA(UpdateSubtitlePreferenceRequest.class));

        service.disableSubtitle();

        verify(fxChannel).send(isA(UpdateSubtitlePreferenceRequest.class));
        assertEquals(SubtitlePreference.Preference.DISABLED, request.get().getPreference().getPreference());
    }

    @Test
    void testReset() {
        service.reset();

        verify(fxChannel).send(isA(ResetSubtitleRequest.class));
    }

    @Test
    void testCleanup() {
        service.cleanup();

        verify(fxChannel).send(isA(CleanSubtitlesDirectoryRequest.class));
    }
}