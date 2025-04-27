package com.github.yoep.popcorn.ui;

import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationArgs;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationArgsRequest;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationArgsResponse;
import com.google.protobuf.Parser;
import javafx.scene.Cursor;
import javafx.scene.Scene;
import javafx.stage.Stage;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;

import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class PopcornTimePreloaderTest {
    @Mock
    private FxChannel fxChannel;
    @InjectMocks
    private PopcornTimePreloader controller;

    @Test
    void testOnMouseDisabled() {
        var stage = mock(Stage.class);
        var scene = mock(Scene.class);
        var request = new AtomicReference<ApplicationArgsRequest>();
        when(fxChannel.send(isA(ApplicationArgsRequest.class), isA(Parser.class))).thenAnswer(invocations -> {
            request.set(invocations.getArgument(0, ApplicationArgsRequest.class));
            return CompletableFuture.completedFuture(ApplicationArgsResponse.newBuilder()
                    .setArgs(ApplicationArgs.newBuilder()
                            .setIsMouseDisabled(true)
                            .build())
                    .build());
        });

        controller.processParameters(stage, scene);

        verify(fxChannel).send(isA(ApplicationArgsRequest.class), isA(Parser.class));
        assertNotNull(request.get(), "expected a request to have been sent");
        verify(scene).setCursor(Cursor.NONE);
    }

    @Test
    void testOnKioskMode() {
        var stage = mock(Stage.class);
        var scene = mock(Scene.class);
        when(fxChannel.send(isA(ApplicationArgsRequest.class), isA(Parser.class))).thenReturn(CompletableFuture.completedFuture(ApplicationArgsResponse.newBuilder()
                .setArgs(ApplicationArgs.newBuilder()
                        .setIsKioskMode(true)
                        .build())
                .build()));

        controller.processParameters(stage, scene);

        verify(fxChannel).send(isA(ApplicationArgsRequest.class), isA(Parser.class));
        verify(stage).setMaximized(true);
    }
}