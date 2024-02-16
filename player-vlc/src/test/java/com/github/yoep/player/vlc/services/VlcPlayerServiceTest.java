package com.github.yoep.player.vlc.services;

import com.github.yoep.player.vlc.VlcListener;
import com.github.yoep.player.vlc.VlcPlayerConstants;
import com.github.yoep.player.vlc.model.VlcState;
import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleMatcher;
import lombok.extern.slf4j.Slf4j;
import mockwebserver3.Dispatcher;
import mockwebserver3.MockResponse;
import mockwebserver3.MockWebServer;
import mockwebserver3.RecordedRequest;
import org.apache.commons.io.IOUtils;
import org.jetbrains.annotations.NotNull;
import org.junit.jupiter.api.AfterAll;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.core.io.ClassPathResource;
import org.springframework.http.HttpHeaders;
import org.springframework.http.MediaType;
import org.springframework.web.reactive.function.client.WebClient;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.util.Optional;
import java.util.Timer;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@Slf4j
@ExtendWith(MockitoExtension.class)
class VlcPlayerServiceTest {
    private static final MockWebServer MOCK_SERVER = new MockWebServer();

    @Mock
    private PlatformProvider platformProvider;
    @Mock
    private SubtitleService subtitleService;
    @Spy
    private WebClient webClient = WebClient.builder().build();
    @InjectMocks
    private VlcPlayerService service;

    @BeforeAll
    static void beforeAll() throws IOException {
        MOCK_SERVER.start(Integer.parseInt(VlcPlayerConstants.PORT));
    }

    @AfterAll
    static void afterAll() {
        try {
            MOCK_SERVER.shutdown();
        } catch (IOException e) {
            log.error("Failed to correctly shutdown the mock server");
        }
    }

    @AfterEach
    void tearDown() {
        service.stop();
    }

    @Test
    void testPlay_whenProcessLaunched_shouldReturnTrue() {
        var url = "my-video-url.mp4";
        var request = mock(PlayRequest.class);
        when(request.getUrl()).thenReturn(url);
        var expectedCommand = "vlc my-video-url.mp4 " + VlcPlayerService.OPTIONS;
        when(platformProvider.launch(expectedCommand)).thenReturn(true);
        MOCK_SERVER.setDispatcher(new Dispatcher() {
            @NotNull
            @Override
            public MockResponse dispatch(@NotNull RecordedRequest recordedRequest) {
                return new MockResponse().setResponseCode(200);
            }
        });

        var result = service.play(request);

        assertTrue(result, "Expected the player to have been started");
        assertNotNull(service.statusTimer, "Expected the status timer to have been created");
    }

    @Test
    void testPlay_whenProcessFailedToLaunch_shouldReturnFalse() {
        var url = "my-video-url.mp4";
        var request = mock(PlayRequest.class);
        when(request.getUrl()).thenReturn(url);
        var expectedCommand = "vlc my-video-url.mp4 " + VlcPlayerService.OPTIONS;
        when(platformProvider.launch(expectedCommand)).thenReturn(false);
        MOCK_SERVER.setDispatcher(new Dispatcher() {
            @NotNull
            @Override
            public MockResponse dispatch(@NotNull RecordedRequest recordedRequest) {
                return new MockResponse().setResponseCode(200);
            }
        });

        var result = service.play(request);

        assertFalse(result, "Expected the player to not have been started");
        assertNull(service.statusTimer, "Expected no status timer to have been created");
    }

    @Test
    void testPlay_whenSubtitleIsActive_shouldAddSubtitlePathToLaunchOption() {
        var url = "my-video-url.mp4";
        var request = mock(PlayRequest.class);
        when(request.getUrl()).thenReturn(url);
        var expectedSubtitleFile = "/tmp/lorem.srt";
        var subtitleInfo = mock(SubtitleInfo.class);
        var expectedCommand = "vlc my-video-url.mp4 " + VlcPlayerService.OPTIONS + " " + VlcPlayerService.SUBTITLE_OPTION + expectedSubtitleFile;
        when(subtitleService.preferredSubtitle()).thenReturn(Optional.of(subtitleInfo));
        when(subtitleService.download(eq(subtitleInfo), isA(SubtitleMatcher.class))).thenReturn(CompletableFuture.completedFuture(expectedSubtitleFile));
        when(platformProvider.launch(isA(String.class))).thenReturn(true);
        MOCK_SERVER.setDispatcher(new Dispatcher() {
            @NotNull
            @Override
            public MockResponse dispatch(@NotNull RecordedRequest recordedRequest) {
                return new MockResponse().setResponseCode(200);
            }
        });

        service.play(request);

        verify(platformProvider).launch(expectedCommand);
    }

    @Test
    void testStop_whenInvoked_shouldStopTheStatusTimer() {
        var timer = mock(Timer.class);

        service.statusTimer = timer;
        service.stop();

        verify(timer).cancel();
        assertNull(service.statusTimer, "Expected the status timer have been stopped");
    }

    @Test
    void testExecuteCommand_whenCommandIsGiven_shouldInvokeCommandOnVlcApi() {
        var command = "myCommand";
        var requestHolder = new AtomicReference<String>();
        var expectedPath = VlcPlayerService.STATUS_PATH + "?command=myCommand";
        MOCK_SERVER.setDispatcher(new Dispatcher() {
            @NotNull
            @Override
            public MockResponse dispatch(@NotNull RecordedRequest recordedRequest) {
                requestHolder.set(recordedRequest.getPath());
                return new MockResponse().setResponseCode(200);
            }
        });

        service.executeCommand(command);
        var requestPath = requestHolder.get();

        assertEquals(expectedPath, requestPath);
    }

    @Test
    void testExecuteCommand_whenErrorIsReturned_shouldNotThrowExceptionUpwards() {
        MOCK_SERVER.setDispatcher(new Dispatcher() {
            @NotNull
            @Override
            public MockResponse dispatch(@NotNull RecordedRequest recordedRequest) {
                return new MockResponse().setResponseCode(500);
            }
        });

        assertDoesNotThrow(() -> service.executeCommand("RandomInvalidCommand"));
    }

    @Test
    void testExecuteCommandAndValue_whenCommandAndValueAreGiven_shouldInvokeCommandOnVlcApi() {
        var command = "myCommand";
        var value = "123";
        var requestHolder = new AtomicReference<String>();
        var expectedPath = VlcPlayerService.STATUS_PATH + "?command=myCommand&val=123";
        MOCK_SERVER.setDispatcher(new Dispatcher() {
            @NotNull
            @Override
            public MockResponse dispatch(@NotNull RecordedRequest recordedRequest) {
                requestHolder.set(recordedRequest.getPath());
                return new MockResponse().setResponseCode(200);
            }
        });

        service.executeCommand(command, value);
        var requestPath = requestHolder.get();

        assertEquals(expectedPath, requestPath);
    }

    @Test
    void testListener_whenTimeHasChanged_shouldInvokeListeners() throws IOException {
        var body = IOUtils.toString(new ClassPathResource("status-changed-event.xml").getInputStream(), StandardCharsets.UTF_8);
        var listener = mock(VlcListener.class);
        when(platformProvider.launch(isA(String.class))).thenReturn(true);
        MOCK_SERVER.setDispatcher(new Dispatcher() {
            @NotNull
            @Override
            public MockResponse dispatch(@NotNull RecordedRequest recordedRequest) {
                var path = recordedRequest.getPath();
                if (path != null && path.equals(VlcPlayerService.STATUS_PATH)) {
                    return new MockResponse()
                            .setBody(body)
                            .setHeader(HttpHeaders.CONTENT_TYPE, MediaType.TEXT_XML_VALUE)
                            .setResponseCode(200);
                } else {
                    return new MockResponse().setResponseCode(404);
                }
            }
        });
        service.addListener(listener);

        service.play(mock(PlayRequest.class));

        verify(listener, timeout(1200)).onTimeChanged(200L);
        verify(listener, timeout(200)).onDurationChanged(56000L);
        verify(listener, timeout(200)).onStateChanged(VlcState.PAUSED);
    }
}