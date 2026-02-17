package com.github.yoep.popcorn.backend.stream;

import com.github.yoep.popcorn.backend.lib.FxCallback;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Stream;
import com.google.protobuf.Parser;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.doAnswer;

@ExtendWith(MockitoExtension.class)
class StreamServerTest {
    @Mock
    private FxChannel fxChannel;
    private StreamServer server;

    private final AtomicReference<FxCallback<Stream.StreamEvent>> eventCallbackHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        doAnswer(invocation -> {
            eventCallbackHolder.set(invocation.getArgument(2, FxCallback.class));
            return null;
        }).when(fxChannel).subscribe(eq(FxChannel.typeFrom(Stream.StreamEvent.class)), isA(Parser.class), isA(FxCallback.class));

        server = new StreamServer(fxChannel);
    }

    @Test
    void testOnStateChangedEvent() {
        var filename = "test.mp4";
        var expectedResult = Stream.StreamState.STREAMING;
        var holder = new AtomicReference<Stream.StreamState>();
        var listener = new StreamListener() {
            @Override
            public void onStateChanged(Stream.StreamState state) {
                holder.set(state);
            }

            @Override
            public void onStatsChanged(Stream.StreamStats stats) {
                // no-op
            }
        };
        server.addListener(filename, listener);

        // invoke the event
        eventCallbackHolder.get().callback(Stream.StreamEvent.newBuilder()
                .setType(Stream.StreamEvent.Type.STATE_CHANGED)
                .setFilename(filename)
                .setState(expectedResult)
                .build());

        // verify the result
        var result = holder.get();
        assertNotNull(result, "expected the state to have been set");
        assertEquals(expectedResult, result);

        // remove the listener
        server.removeListener(filename, listener);

        // invoke the event
        eventCallbackHolder.get().callback(Stream.StreamEvent.newBuilder()
                .setType(Stream.StreamEvent.Type.STATE_CHANGED)
                .setFilename(filename)
                .setState(Stream.StreamState.STOPPED)
                .build());
        assertEquals(expectedResult, result, "expected the state to remain unchanged");
    }

    @Test
    void testOnStatsChangedEvent() {
        var filename = "test.mp4";
        var expectedResult = Stream.StreamStats.newBuilder()
                .setProgress(10.0f)
                .setConnections(8)
                .setDownloaded(100L)
                .setTotalSize(1000L)
                .build();
        var holder = new AtomicReference<Stream.StreamStats>();
        var listener = new StreamListener() {
            @Override
            public void onStateChanged(Stream.StreamState state) {
                // no-op
            }

            @Override
            public void onStatsChanged(Stream.StreamStats stats) {
                holder.set(stats);
            }
        };
        server.addListener(filename, listener);

        // invoke the event
        eventCallbackHolder.get().callback(Stream.StreamEvent.newBuilder()
                .setType(Stream.StreamEvent.Type.STATS_CHANGED)
                .setFilename(filename)
                .setStats(expectedResult)
                .build());

        var result = holder.get();
        assertNotNull(result, "expected the stats to have been set");
        assertEquals(expectedResult, result);
    }
}