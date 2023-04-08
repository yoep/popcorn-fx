package com.github.yoep.popcorn.backend.logging;

import ch.qos.logback.classic.Level;
import ch.qos.logback.classic.spi.ILoggingEvent;
import com.github.yoep.popcorn.backend.FxLib;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class LoggingBridgeTest {
    @Mock
    private FxLib fxLib;
    @InjectMocks
    private LoggingBridge bridge;

    @Test
    void testAppendTrace() {
        var event = mock(ILoggingEvent.class);
        when(event.getFormattedMessage()).thenReturn("lorem");
        when(event.getLevel()).thenReturn(Level.TRACE);
        when(event.getLoggerName()).thenReturn("com.github.yoep.popcorn.backend.logging.LoggingBridgeTest");

        bridge.append(event);

        verify(fxLib).log("java::com.github.yoep.popcorn.backend.logging.LoggingBridgeTest", "lorem", LogLevel.TRACE);
    }

    @Test
    void testAppendDebug() {
        var event = mock(ILoggingEvent.class);
        when(event.getFormattedMessage()).thenReturn("ipsum");
        when(event.getLevel()).thenReturn(Level.DEBUG);
        when(event.getLoggerName()).thenReturn("LoggingBridgeTest");

        bridge.append(event);

        verify(fxLib).log("java::LoggingBridgeTest", "ipsum", LogLevel.DEBUG);
    }

    @Test
    void testAppendInfo() {
        var event = mock(ILoggingEvent.class);
        when(event.getFormattedMessage()).thenReturn("dolor");
        when(event.getLevel()).thenReturn(Level.INFO);
        when(event.getLoggerName()).thenReturn("LoggingBridgeTest");

        bridge.append(event);

        verify(fxLib).log("java::LoggingBridgeTest", "dolor", LogLevel.INFO);
    }
}