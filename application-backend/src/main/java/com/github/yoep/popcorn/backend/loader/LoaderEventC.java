package com.github.yoep.popcorn.backend.loader;

import com.sun.jna.FromNativeContext;
import com.sun.jna.NativeMapped;
import com.sun.jna.Structure;
import com.sun.jna.Union;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

import java.io.Closeable;
import java.util.Arrays;
import java.util.Optional;

@Getter
@ToString
@Structure.FieldOrder({"tag", "union"})
public class LoaderEventC extends Structure implements Closeable {
    public static class ByValue extends LoaderEventC implements Structure.ByValue {
        @Override
        public void close() {
            super.close();

        }
    }

    public Tag tag;
    public LoaderEventCUnion.ByValue union;

    @Override
    public void read() {
        super.read();
        updateUnionType();
        union.read();
    }

    @Override
    public void close() {
        setAutoSynch(false);
        getUnion().close();
    }

    private void updateUnionType() {
        switch (tag) {
            case LOADING_STARTED -> union.setType(LoadingStarted_Body.class);
            case STATE_CHANGED -> union.setType(StateChanged_Body.class);
            case PROGRESS_CHANGED -> union.setType(ProgressChanged_Body.class);
            case LOADING_ERROR -> union.setType(LoadingError_Body.class);
        }
    }

    @Getter
    @ToString
    @FieldOrder({"handle", "startedEvent"})
    public static class LoadingStarted_Body extends Structure implements Closeable {
        public Long handle;
        public LoadingStartedEventC.ByValue startedEvent;

        @Override
        public void close() {
            setAutoSynch(false);
            Optional.ofNullable(startedEvent)
                    .ifPresent(LoadingStartedEventC::close);
        }
    }

    @Getter
    @ToString
    @FieldOrder({"handle", "state"})
    public static class StateChanged_Body extends Structure implements Closeable {
        public Long handle;
        public LoaderState state;

        @Override
        public void close() {
            setAutoSynch(false);
        }
    }

    @Getter
    @ToString
    @FieldOrder({"handle", "loadingProgress"})
    public static class ProgressChanged_Body extends Structure implements Closeable {
        public Long handle;
        public LoadingProgress.ByValue loadingProgress;

        @Override
        public void close() {
            setAutoSynch(false);
            Optional.ofNullable(loadingProgress)
                    .ifPresent(LoadingProgress::close);
        }
    }

    @Getter
    @ToString
    @FieldOrder({"handle", "error"})
    public static class LoadingError_Body extends Structure implements Closeable {
        public Long handle;
        public LoadingErrorC error;

        @Override
        public void close() {
            setAutoSynch(false);
        }
    }

    @Getter
    @ToString
    @EqualsAndHashCode(callSuper = false)
    public static class LoaderEventCUnion extends Union implements Closeable {
        public static class ByValue extends LoaderEventCUnion implements Union.ByValue {
        }

        public LoadingStarted_Body loadingStarted_body;
        public StateChanged_Body stateChanged_body;
        public ProgressChanged_Body progressChanged_body;
        public LoadingError_Body loadingError_body;

        @Override
        public void close() {
            setAutoSynch(false);
            Optional.ofNullable(loadingStarted_body)
                    .ifPresent(LoadingStarted_Body::close);
            Optional.ofNullable(stateChanged_body)
                    .ifPresent(StateChanged_Body::close);
            Optional.ofNullable(progressChanged_body)
                    .ifPresent(ProgressChanged_Body::close);
            Optional.ofNullable(loadingError_body)
                    .ifPresent(LoadingError_Body::close);
        }
    }

    public enum Tag implements NativeMapped {
        LOADING_STARTED,
        STATE_CHANGED,
        PROGRESS_CHANGED,
        LOADING_ERROR;

        @Override
        public Object fromNative(Object nativeValue, FromNativeContext context) {
            return Arrays.stream(values())
                    .filter(e -> e.ordinal() == (int) nativeValue)
                    .findFirst()
                    .orElse(null);
        }

        @Override
        public Object toNative() {
            return ordinal();
        }

        @Override
        public Class<?> nativeType() {
            return Integer.class;
        }
    }
}
