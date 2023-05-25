package com.github.yoep.popcorn.backend.logging;

import ch.qos.logback.classic.Level;
import ch.qos.logback.classic.spi.ILoggingEvent;
import ch.qos.logback.classic.spi.ThrowableProxy;
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

        verify(fxLib).log(LoggingBridge.PREFIX + "::com.github.yoep.popcorn.backend.logging.LoggingBridgeTest", "lorem", LogLevel.TRACE);
    }

    @Test
    void testAppendDebug() {
        var event = mock(ILoggingEvent.class);
        when(event.getFormattedMessage()).thenReturn("ipsum");
        when(event.getLevel()).thenReturn(Level.DEBUG);
        when(event.getLoggerName()).thenReturn("LoggingBridgeTest");

        bridge.append(event);

        verify(fxLib).log(LoggingBridge.PREFIX + "::LoggingBridgeTest", "ipsum", LogLevel.DEBUG);
    }

    @Test
    void testAppendInfo() {
        var event = mock(ILoggingEvent.class);
        when(event.getFormattedMessage()).thenReturn("dolor");
        when(event.getLevel()).thenReturn(Level.INFO);
        when(event.getLoggerName()).thenReturn("LoggingBridgeTest");

        bridge.append(event);

        verify(fxLib).log(LoggingBridge.PREFIX + "::LoggingBridgeTest", "dolor", LogLevel.INFO);
    }

    @Test
    void testAppendWarn() {
        var event = mock(ILoggingEvent.class);
        when(event.getFormattedMessage()).thenReturn("sit");
        when(event.getLevel()).thenReturn(Level.WARN);
        when(event.getLoggerName()).thenReturn("LoggingBridgeTest");

        bridge.append(event);

        verify(fxLib).log(LoggingBridge.PREFIX + "::LoggingBridgeTest", "sit", LogLevel.WARN);
    }

    @Test
    void testAppendError() {
        var event = mock(ILoggingEvent.class);
        when(event.getFormattedMessage()).thenReturn("amit");
        when(event.getLevel()).thenReturn(Level.ERROR);
        when(event.getLoggerName()).thenReturn("LoggingBridgeTest");
        when(event.getThrowableProxy()).thenReturn(new ThrowableProxy(new RuntimeException("my runtime exception")));

        bridge.append(event);

        verify(fxLib).log(eq(LoggingBridge.PREFIX + "::LoggingBridgeTest"), startsWith("amit\njava.lang.RuntimeException: my runtime exception"), eq(LogLevel.ERROR));
    }
}