package com.github.yoep.popcorn.backend.logging;

import ch.qos.logback.classic.Level;
import ch.qos.logback.classic.spi.ILoggingEvent;
import ch.qos.logback.classic.spi.IThrowableProxy;
import ch.qos.logback.core.AppenderBase;
import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.lib.FxLibInstance;

public class LoggingBridge extends AppenderBase<ILoggingEvent> {
    static final String PREFIX = "jvm";

    private final FxLib fxLib;

    public LoggingBridge() {
        this.fxLib = FxLibInstance.INSTANCE.get();
    }

    public LoggingBridge(FxLib fxLib) {
        this.fxLib = fxLib;
    }

    @Override
    protected void append(ILoggingEvent event) {
        var message = event.getFormattedMessage();

        if (event.getThrowableProxy() != null) {
            message += "\n";
            message += convertThrowableProxyToString(event.getThrowableProxy());
        }

        fxLib.log(PREFIX + "::" + event.getLoggerName(), message, map(event.getLevel()));
    }

    private static LogLevel map(Level level) {
        if (level == Level.TRACE) {
            return LogLevel.TRACE;
        }
        if (level == Level.DEBUG) {
            return LogLevel.DEBUG;
        }
        if (level == Level.INFO) {
            return LogLevel.INFO;
        }
        if (level == Level.WARN) {
            return LogLevel.WARN;
        }
        if (level == Level.ERROR) {
            return LogLevel.ERROR;
        }

        return LogLevel.OFF;
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
