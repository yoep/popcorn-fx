package com.github.yoep.popcorn.backend.logging;

import ch.qos.logback.classic.Level;
import ch.qos.logback.classic.spi.ILoggingEvent;
import ch.qos.logback.classic.spi.IThrowableProxy;
import ch.qos.logback.core.AppenderBase;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.LogRequest;
import lombok.Data;
import lombok.EqualsAndHashCode;

import java.util.concurrent.atomic.AtomicReference;

@Data
@EqualsAndHashCode(callSuper = false)
public class LoggingBridge extends AppenderBase<ILoggingEvent> {
    public static final AtomicReference<LoggingBridge> INSTANCE = new AtomicReference<>();
    static final String PREFIX = "jvm";

    private FxChannel fxChannel;

    public LoggingBridge() {
        INSTANCE.set(this);
    }

    @Override
    protected void append(ILoggingEvent event) {
        var message = event.getFormattedMessage();

        if (event.getThrowableProxy() != null) {
            message += "\n";
            message += convertThrowableProxyToString(event.getThrowableProxy());
        }

        if (fxChannel != null) {
            fxChannel.send(LogRequest.newBuilder()
                    .setLevel(map_ipc(event.getLevel()))
                    .setTarget(PREFIX + "::" + event.getLoggerName())
                    .setMessage(message)
                    .build());
        }
    }

    private static LogRequest.LogLevel map_ipc(Level level) {
        if (level == Level.TRACE) {
            return LogRequest.LogLevel.TRACE;
        }
        if (level == Level.DEBUG) {
            return LogRequest.LogLevel.DEBUG;
        }
        if (level == Level.INFO) {
            return LogRequest.LogLevel.INFO;
        }
        if (level == Level.WARN) {
            return LogRequest.LogLevel.WARN;
        }
        if (level == Level.ERROR) {
            return LogRequest.LogLevel.ERROR;
        }

        return LogRequest.LogLevel.OFF;
    }

    private static String convertThrowableProxyToString(IThrowableProxy throwableProxy) {
        var stringBuilder = new StringBuilder();
        appendThrowableProxy(stringBuilder, throwableProxy, "");
        return stringBuilder.toString();
    }

    private static void appendThrowableProxy(StringBuilder stringBuilder, IThrowableProxy throwableProxy, String indent) {
        stringBuilder.append(indent)
                .append(throwableProxy.getClassName())
                .append(": ")
                .append(throwableProxy.getMessage())
                .append(System.lineSeparator());

        for (var stackTraceElementProxy : throwableProxy.getStackTraceElementProxyArray()) {
            stringBuilder.append(indent)
                    .append("\tat ")
                    .append(stackTraceElementProxy.getSTEAsString())
                    .append(System.lineSeparator());
        }

        var cause = throwableProxy.getCause();
        if (cause != null) {
            stringBuilder.append(indent)
                    .append("Caused by: ")
                    .append(System.lineSeparator());
            appendThrowableProxy(stringBuilder, cause, indent + "\t");
        }
    }
}
