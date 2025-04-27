package com.github.yoep.popcorn.backend.media.providers;

import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.*;
import com.github.yoep.popcorn.backend.media.MovieOverview;
import com.google.protobuf.Parser;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class ProviderServiceImplTest {
    @Mock
    private FxChannel fxChannel;
    @InjectMocks
    private ProviderServiceImpl service;

    @Test
    void testGetPage_WithoutKeywords() {
        var category = Media.Category.MOVIES;
        var genre = Media.Genre.newBuilder()
                .setKey("all")
                .build();
        var sortBy = Media.SortBy.newBuilder()
                .setKey("watched")
                .build();
        var page = 1;
        var request = new AtomicReference<GetMediaItemsRequest>();
        when(fxChannel.send(isA(GetMediaItemsRequest.class), isA(Parser.class))).thenAnswer(invocations -> {
            request.set(invocations.getArgument(0, GetMediaItemsRequest.class));
            return CompletableFuture.completedFuture(GetMediaItemsResponse.newBuilder()
                    .addItems(Media.Item.newBuilder()
                            .setType(FxChannel.typeFrom(Media.MovieOverview.class))
                            .setMovieOverview(createMovieMedia())
                            .build())
                    .build());
        });

        var result = service.getPage(category, genre, sortBy, page).resultNow();

        verify(fxChannel).send(isA(GetMediaItemsRequest.class), isA(Parser.class));
        assertEquals(category, request.get().getCategory());
        assertEquals(genre.getKey(), request.get().getGenre().getKey());
        assertEquals(sortBy.getKey(), request.get().getSortBy().getKey());
        assertEquals(page, request.get().getPage());

        var resultItem = result.getFirst();
        assertNotNull(resultItem, "expected a media item to have been returned");
        assertEquals("tt000001", resultItem.id());
        assertEquals("MyMovie", resultItem.title());
    }

    @Test
    void testGetPage_WithKeywords() {
        var category = Media.Category.SERIES;
        var genre = Media.Genre.newBuilder()
                .setKey("all")
                .build();
        var sortBy = Media.SortBy.newBuilder()
                .setKey("year")
                .build();
        var page = 2;
        var keywords = "Lorem ipsum dolor";
        var request = new AtomicReference<GetMediaItemsRequest>();
        when(fxChannel.send(isA(GetMediaItemsRequest.class), isA(Parser.class))).thenAnswer(invocations -> {
            request.set(invocations.getArgument(0, GetMediaItemsRequest.class));
            return CompletableFuture.completedFuture(GetMediaItemsResponse.newBuilder()
                    .addItems(Media.Item.newBuilder()
                            .setType(FxChannel.typeFrom(Media.ShowOverview.class))
                            .setShowOverview(createShowMedia())
                            .build())
                    .build());
        });

        var result = service.getPage(category, genre, sortBy, page, keywords).resultNow();

        verify(fxChannel).send(isA(GetMediaItemsRequest.class), isA(Parser.class));
        assertEquals(category, request.get().getCategory());
        assertEquals(genre.getKey(), request.get().getGenre().getKey());
        assertEquals(sortBy.getKey(), request.get().getSortBy().getKey());
        assertEquals(page, request.get().getPage());
        assertEquals(keywords, request.get().getKeywords());

        var resultItem = result.getFirst();
        assertNotNull(resultItem, "expected a media item to have been returned");
        assertEquals("tt000002", resultItem.id());
        assertEquals("MySerie", resultItem.title());
    }

    @Test
    void testRetrieveDetails() {
        var movie = new MovieOverview(createMovieMedia());
        var request = new AtomicReference<GetMediaDetailsRequest>();
        when(fxChannel.send(isA(GetMediaDetailsRequest.class), isA(Parser.class))).thenAnswer(invocations -> {
            request.set(invocations.getArgument(0, GetMediaDetailsRequest.class));
            return CompletableFuture.completedFuture(GetMediaDetailsResponse.newBuilder()
                    .setItem(Media.Item.newBuilder()
                            .setType(FxChannel.typeFrom(Media.MovieDetails.class))
                            .setMovieDetails(createMovieDetailsMedia())
                            .build())
                    .build());
        });

        var result = service.retrieveDetails(movie).resultNow();

        verify(fxChannel).send(isA(GetMediaDetailsRequest.class), isA(Parser.class));
        assertEquals(FxChannel.typeFrom(Media.MovieOverview.class), request.get().getItem().getType());

        assertNotNull(result, "expected to have retrieved a media item");
        assertEquals("Lorem ipsum dolor", result.synopsis());
    }

    @Test
    void testResetApiAvailability() {
        var category = Media.Category.MOVIES;
        var request = new AtomicReference<ResetProviderApiRequest>();
        doAnswer(invocations -> {
            request.set(invocations.getArgument(0, ResetProviderApiRequest.class));
            return null;
        }).when(fxChannel).send(isA(ResetProviderApiRequest.class));

        service.resetApiAvailability(category);

        verify(fxChannel).send(isA(ResetProviderApiRequest.class));
        assertEquals(category, request.get().getCategory());
    }

    private static Media.MovieOverview createMovieMedia() {
        return Media.MovieOverview.newBuilder()
                .setImdbId("tt000001")
                .setTitle("MyMovie")
                .build();
    }


    private static Media.MovieDetails createMovieDetailsMedia() {
        return Media.MovieDetails.newBuilder()
                .setImdbId("tt000001")
                .setTitle("MyMovie")
                .setSynopsis("Lorem ipsum dolor")
                .setRuntime(120)
                .build();
    }

    private static Media.ShowOverview createShowMedia() {
        return Media.ShowOverview.newBuilder()
                .setImdbId("tt000002")
                .setTitle("MySerie")
                .build();
    }
}