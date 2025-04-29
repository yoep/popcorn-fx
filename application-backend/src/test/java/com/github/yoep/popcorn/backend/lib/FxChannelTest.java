package com.github.yoep.popcorn.backend.lib;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.*;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.io.IOException;
import java.util.Queue;
import java.util.concurrent.*;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class FxChannelTest {
    @Mock
    private FxLib fxLib;
    private FxChannel channel;

    private final Queue<FxMessage> messages = new ConcurrentLinkedQueue<>();

    @BeforeEach
    void setUp() {
        var executor = Executors.newCachedThreadPool(e -> new Thread(e, "popcorn-fx-test"));
        when(fxLib.receive()).thenAnswer(invocation -> {
            while (true) {
                if (!messages.isEmpty()) {
                    return messages.poll();
                }
                Thread.yield();
            }
        });
        channel = new FxChannel(fxLib, executor);
    }

    @Test
    void testSend_MessageWithResponse() throws ExecutionException, InterruptedException, TimeoutException {
        var message = new AtomicReference<FxMessage>();
        doAnswer(invocations -> {
            message.set(invocations.getArgument(0, FxMessage.class));
            return null;
        }).when(fxLib).send(isA(FxMessage.class));

        var future = channel.send(GetPlayerVolumeRequest.getDefaultInstance(), GetPlayerVolumeResponse.parser());
        messages.add(FxMessage.newBuilder()
                .setType(FxChannel.typeFrom(GetPlayerVolumeResponse.class))
                .setSequenceId(1)
                .setReplyTo(1)
                .setPayload(GetPlayerVolumeResponse.newBuilder()
                        .setVolume(30)
                        .build()
                        .toByteString())
                .build());

        var response = future.get(2, TimeUnit.SECONDS);

        verify(fxLib).send(isA(FxMessage.class));
        assertEquals(FxChannel.typeFrom(GetPlayerVolumeRequest.class), message.get().getType());

        assertNotNull(response, "expected to have received a response");
        assertEquals(30, response.getVolume());
    }

    @Test
    void testSend_MessageWithoutResponse() {
        var message = new AtomicReference<FxMessage>();
        doAnswer(invocations -> {
            message.set(invocations.getArgument(0, FxMessage.class));
            return null;
        }).when(fxLib).send(isA(FxMessage.class));

        channel.send(LogRequest.getDefaultInstance());

        verify(fxLib).send(isA(FxMessage.class));
        assertEquals(FxChannel.typeFrom(LogRequest.class), message.get().getType());
    }

    @Test
    void testSend_withReplyId() {
        var replyTo = 13;
        var message = new AtomicReference<FxMessage>();
        doAnswer(invocations -> {
            message.set(invocations.getArgument(0, FxMessage.class));
            return null;
        }).when(fxLib).send(isA(FxMessage.class));

        channel.send(LogRequest.getDefaultInstance(), replyTo);

        verify(fxLib).send(isA(FxMessage.class));
        assertEquals(FxChannel.typeFrom(LogRequest.class), message.get().getType());
        assertEquals(replyTo, message.get().getReplyTo());
    }

    @Test
    void testSubscribe() {
        var callback = mock(FxCallback.class);

        channel.subscribe(FxChannel.typeFrom(Event.class), Event.parser(), callback);
        messages.add(FxMessage.newBuilder()
                .setType(FxChannel.typeFrom(Event.class))
                .setSequenceId(1)
                .setPayload(Event.newBuilder().build().toByteString())
                .build());

        verify(callback, timeout(250)).callback(isA(Event.class));
    }

    @Test
    void testClose() throws IOException {
        channel.close();

        verify(fxLib).close();
        assertEquals(false, channel.running.get());
    }
}