package com.github.yoep.popcorn.backend.logging;

import ch.qos.logback.classic.Level;
import ch.qos.logback.classic.spi.ILoggingEvent;
import ch.qos.logback.core.AppenderBase;
import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.lib.FxLibInstance;

public class LoggingBridge extends AppenderBase<ILoggingEvent> {
    private final FxLib fxLib;

    public LoggingBridge() {
        this.fxLib = FxLibInstance.INSTANCE.get();
    }

    public LoggingBridge(FxLib fxLib) {
        this.fxLib = fxLib;
    }

    @Override
    protected void append(ILoggingEvent event) {
        fxLib.log("java::" + event.getLoggerName(), event.getFormattedMessage(), map(event.getLevel()));
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
}
