package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.*;
import com.github.yoep.popcorn.backend.media.ShowOverview;
import com.google.protobuf.ByteString;
import com.google.protobuf.Parser;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class ImageServiceTest {
    @Mock
    private FxChannel fxChannel;
    @Mock
    private ExecutorService executorService;
    @InjectMocks
    private ImageService imageService;

    @BeforeEach
    void setUp() {
        lenient().doAnswer(invocation -> {
            var runnable = invocation.getArgument(0, Runnable.class);
            runnable.run();
            return null;
        }).when(executorService).execute(isA(Runnable.class));
    }

    @Test
    void testGetPosterPlaceholder() {
        var data = ImageServiceTest.class.getResourceAsStream("/posterholder.png");
        var request = new AtomicReference<GetPosterPlaceholderRequest>();
        when(fxChannel.send(isA(GetPosterPlaceholderRequest.class), isA(Parser.class))).thenAnswer(invocations -> {
            request.set(invocations.getArgument(0, GetPosterPlaceholderRequest.class));
            return CompletableFuture.completedFuture(GetPosterPlaceholderResponse.newBuilder()
                    .setImage(Image.newBuilder()
                            .setData(ByteString.readFrom(data))
                            .build())
                    .build());
        });

        var result = imageService.getPosterPlaceholder().resultNow();

        verify(fxChannel).send(isA(GetPosterPlaceholderRequest.class), isA(Parser.class));
        assertNotNull(request.get(), "expected a request to have been sent");
        assertNotNull(result);
    }

    @Test
    void testGetArtworkPlaceholder() {
        var data = ImageServiceTest.class.getResourceAsStream("/posterholder.png");
        var request = new AtomicReference<GetArtworkPlaceholderRequest>();
        when(fxChannel.send(isA(GetArtworkPlaceholderRequest.class), isA(Parser.class))).thenAnswer(invocations -> {
            request.set(invocations.getArgument(0, GetArtworkPlaceholderRequest.class));
            return CompletableFuture.completedFuture(GetArtworkPlaceholderResponse.newBuilder()
                    .setImage(Image.newBuilder()
                            .setData(ByteString.readFrom(data))
                            .build())
                    .build());
        });

        var result = imageService.getArtPlaceholder().resultNow();

        verify(fxChannel).send(isA(GetArtworkPlaceholderRequest.class), isA(Parser.class));
        assertNotNull(result);
    }

    @Test
    void testLoadFanart() {
        var url = "http://my-fanart-url.com";
        var media = new ShowOverview(Media.ShowOverview.newBuilder()
                .setImages(Media.Images.newBuilder()
                        .setFanart(url)
                        .build())
                .build());
        var data = ImageServiceTest.class.getResourceAsStream("/posterholder.png");
        var request = new AtomicReference<GetFanartRequest>();
        when(fxChannel.send(isA(GetFanartRequest.class), isA(Parser.class))).thenAnswer(invocations -> {
            request.set(invocations.getArgument(0, GetFanartRequest.class));
            return CompletableFuture.completedFuture(GetFanartResponse.newBuilder()
                    .setImage(Image.newBuilder()
                            .setData(ByteString.readFrom(data))
                            .build())
                    .build());
        });

        var result = imageService.loadFanart(media).resultNow();

        verify(fxChannel).send(isA(GetFanartRequest.class), isA(Parser.class));
        assertEquals(media.proto(), request.get().getMedia().getShowOverview());
        assertNotNull(result);
    }

    @Test
    void testLoadPoster() {
        var media = new ShowOverview(Media.ShowOverview.newBuilder()
                .setImages(Media.Images.newBuilder()
                        .setPoster("http://my-poster-url.com")
                        .build())
                .build());
        var data = ImageServiceTest.class.getResourceAsStream("/posterholder.png");
        var request = new AtomicReference<GetPosterRequest>();
        when(fxChannel.send(isA(GetPosterRequest.class), isA(Parser.class))).thenAnswer(invocations -> {
            request.set(invocations.getArgument(0, GetPosterRequest.class));
            return CompletableFuture.completedFuture(GetPosterResponse.newBuilder()
                    .setImage(Image.newBuilder()
                            .setData(ByteString.readFrom(data))
                            .build())
                    .build());
        });

        var result = imageService.loadPoster(media).resultNow();

        verify(fxChannel).send(isA(GetPosterRequest.class), isA(Parser.class));
        assertEquals(media.proto(), request.get().getMedia().getShowOverview());
        assertNotNull(result);
    }

    @Test
    void testLoad() {
        var url = "http://localhost/image.png";
        var request = new AtomicReference<GetImageRequest>();
        when(fxChannel.send(isA(GetImageRequest.class), isA(Parser.class))).thenAnswer(invocations -> {
            request.set(invocations.getArgument(0, GetImageRequest.class));
            return CompletableFuture.completedFuture(GetImageResponse.newBuilder()
                    .setResult(Response.Result.OK)
                    .setImage(Image.newBuilder().setData(ByteString.EMPTY))
                    .build());
        });

        var image = imageService.load(url).resultNow();

        verify(fxChannel).send(isA(GetImageRequest.class), isA(Parser.class));
        assertEquals(url, request.get().getUrl());
        assertNotNull(image, "expected a image to have been return");
    }
}
