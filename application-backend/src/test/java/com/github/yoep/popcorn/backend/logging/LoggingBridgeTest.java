package com.github.yoep.popcorn.backend.logging;

import ch.qos.logback.classic.Level;
import ch.qos.logback.classic.spi.ILoggingEvent;
import ch.qos.logback.classic.spi.ThrowableProxy;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.LogRequest;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class LoggingBridgeTest {
    @Mock
    private FxChannel fxChannel;
    @InjectMocks
    private LoggingBridge bridge;

    @Test
    void testAppendTrace() {
        var event = mock(ILoggingEvent.class);
        var messageHolder = new AtomicReference<LogRequest>();
        doAnswer(invocations -> {
            messageHolder.set(invocations.getArgument(0, LogRequest.class));
            return null;
        }).when(fxChannel).send(isA(LogRequest.class));
        when(event.getFormattedMessage()).thenReturn("lorem");
        when(event.getLevel()).thenReturn(Level.TRACE);
        when(event.getLoggerName()).thenReturn("com.github.yoep.popcorn.backend.logging.LoggingBridgeTest");

        bridge.append(event);

        verify(fxChannel).send(isA(LogRequest.class));
        var result = messageHolder.get();
        assertEquals(LoggingBridge.PREFIX + "::com.github.yoep.popcorn.backend.logging.LoggingBridgeTest", result.getTarget());
        assertEquals("lorem", result.getMessage());
        assertEquals(LogRequest.LogLevel.TRACE, result.getLevel());
    }

    @Test
    void testAppendDebug() {
        var event = mock(ILoggingEvent.class);
        var messageHolder = new AtomicReference<LogRequest>();
        doAnswer(invocations -> {
            messageHolder.set(invocations.getArgument(0, LogRequest.class));
            return null;
        }).when(fxChannel).send(isA(LogRequest.class));
        when(event.getFormattedMessage()).thenReturn("ipsum");
        when(event.getLevel()).thenReturn(Level.DEBUG);
        when(event.getLoggerName()).thenReturn("LoggingBridgeTest");

        bridge.append(event);

        verify(fxChannel).send(isA(LogRequest.class));
        var result = messageHolder.get();
        assertEquals(LoggingBridge.PREFIX + "::LoggingBridgeTest", result.getTarget());
        assertEquals("ipsum", result.getMessage());
        assertEquals(LogRequest.LogLevel.DEBUG, result.getLevel());
    }

    @Test
    void testAppendInfo() {
        var event = mock(ILoggingEvent.class);
        var messageHolder = new AtomicReference<LogRequest>();
        doAnswer(invocations -> {
            messageHolder.set(invocations.getArgument(0, LogRequest.class));
            return null;
        }).when(fxChannel).send(isA(LogRequest.class));
        when(event.getFormattedMessage()).thenReturn("dolor");
        when(event.getLevel()).thenReturn(Level.INFO);
        when(event.getLoggerName()).thenReturn("LoggingBridgeTest");

        bridge.append(event);

        verify(fxChannel).send(isA(LogRequest.class));
        var result = messageHolder.get();
        assertEquals(LoggingBridge.PREFIX + "::LoggingBridgeTest", result.getTarget());
        assertEquals("dolor", result.getMessage());
        assertEquals(LogRequest.LogLevel.INFO, result.getLevel());
    }

    @Test
    void testAppendWarn() {
        var event = mock(ILoggingEvent.class);
        var messageHolder = new AtomicReference<LogRequest>();
        doAnswer(invocations -> {
            messageHolder.set(invocations.getArgument(0, LogRequest.class));
            return null;
        }).when(fxChannel).send(isA(LogRequest.class));
        when(event.getFormattedMessage()).thenReturn("sit");
        when(event.getLevel()).thenReturn(Level.WARN);
        when(event.getLoggerName()).thenReturn("LoggingBridgeTest");

        bridge.append(event);

        verify(fxChannel).send(isA(LogRequest.class));
        var result = messageHolder.get();
        assertEquals(LoggingBridge.PREFIX + "::LoggingBridgeTest", result.getTarget());
        assertEquals("sit", result.getMessage());
        assertEquals(LogRequest.LogLevel.WARN, result.getLevel());
    }

    @Test
    void testAppendError() {
        var event = mock(ILoggingEvent.class);
        var messageHolder = new AtomicReference<LogRequest>();
        doAnswer(invocations -> {
            messageHolder.set(invocations.getArgument(0, LogRequest.class));
            return null;
        }).when(fxChannel).send(isA(LogRequest.class));
        when(event.getFormattedMessage()).thenReturn("amit");
        when(event.getLevel()).thenReturn(Level.ERROR);
        when(event.getLoggerName()).thenReturn("LoggingBridgeTest");
        when(event.getThrowableProxy()).thenReturn(new ThrowableProxy(new RuntimeException("my runtime exception")));

        bridge.append(event);

        verify(fxChannel).send(isA(LogRequest.class));
        var result = messageHolder.get();
        assertEquals(LoggingBridge.PREFIX + "::LoggingBridgeTest", result.getTarget());
        assertTrue(result.getMessage().startsWith("amit\njava.lang.RuntimeException: my runtime exception"));
        assertEquals(LogRequest.LogLevel.ERROR, result.getLevel());
    }
}