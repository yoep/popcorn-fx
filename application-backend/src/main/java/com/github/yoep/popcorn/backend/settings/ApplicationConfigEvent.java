package com.github.yoep.popcorn.backend.settings;

import com.github.yoep.popcorn.backend.settings.models.*;
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
public class ApplicationConfigEvent extends Structure implements Closeable {
    public static class ByValue extends ApplicationConfigEvent implements Structure.ByValue {
    }

    public Tag tag;
    public ApplicationConfigEventUnion.ByValue union;

    @Override
    public void read() {
        super.read();
        switch (tag) {
            case SUBTITLE_SETTINGS_CHANGED -> union.setType(ApplicationConfigEvent.SubtitleSettingsChanged_Body.class);
            case TORRENT_SETTINGS_CHANGED -> union.setType(ApplicationConfigEvent.TorrentSettingsChanged_Body.class);
            case UI_SETTINGS_CHANGED -> union.setType(ApplicationConfigEvent.UiSettingsChanged_Body.class);
            case SERVER_SETTINGS_CHANGED -> union.setType(ApplicationConfigEvent.ServerSettingsChanged_Body.class);
            case PLAYBACK_SETTINGS_CHANGED -> union.setType(ApplicationConfigEvent.PlaybackSettingsChanged_Body.class);
        }
        union.read();
    }

    @Override
    public void close() {
        setAutoSynch(false);
        getUnion().close();
    }

    @Getter
    @ToString
    @FieldOrder({"settings"})
    public static class SubtitleSettingsChanged_Body extends Structure implements Closeable {
        public SubtitleSettings settings;

        @Override
        public void close() {
            setAutoSynch(false);
        }
    }

    @Getter
    @ToString
    @FieldOrder({"settings"})
    public static class TorrentSettingsChanged_Body extends Structure implements Closeable {
        public TorrentSettings settings;

        @Override
        public void close() {
            setAutoSynch(false);
        }
    }

    @Getter
    @ToString
    @FieldOrder({"settings"})
    public static class UiSettingsChanged_Body extends Structure implements Closeable {
        public UISettings settings;

        @Override
        public void close() {
            setAutoSynch(false);
        }
    }

    @Getter
    @ToString
    @FieldOrder({"settings"})
    public static class ServerSettingsChanged_Body extends Structure implements Closeable {
        public ServerSettings settings;

        @Override
        public void close() {
            setAutoSynch(false);
        }
    }

    @Getter
    @ToString
    @FieldOrder({"settings"})
    public static class PlaybackSettingsChanged_Body extends Structure implements Closeable {
        public PlaybackSettings settings;

        @Override
        public void close() {
            setAutoSynch(false);
        }
    }

    @Getter
    @ToString
    @FieldOrder({"settings"})
    public static class TrackingSettingsChanged_Body extends Structure implements Closeable {
        public TrackingSettings settings;

        @Override
        public void close() {
            setAutoSynch(false);
        }
    }

    @Getter
    @ToString
    @EqualsAndHashCode(callSuper = false)
    public static class ApplicationConfigEventUnion extends Union implements Closeable {
        public static class ByValue extends ApplicationConfigEventUnion implements Union.ByValue {
        }

        public SubtitleSettingsChanged_Body subtitleSettingsChanged_body;
        public TorrentSettingsChanged_Body torrentSettingsChanged_body;
        public UiSettingsChanged_Body uiSettingsChanged_body;
        public ServerSettingsChanged_Body serverSettingsChanged_body;
        public PlaybackSettingsChanged_Body playbackSettingsChanged_body;
        public TrackingSettingsChanged_Body trackingSettingsChanged_body;

        @Override
        public void close() {
            setAutoSynch(false);
            Optional.ofNullable(subtitleSettingsChanged_body)
                    .ifPresent(SubtitleSettingsChanged_Body::close);
            Optional.ofNullable(torrentSettingsChanged_body)
                    .ifPresent(TorrentSettingsChanged_Body::close);
            Optional.ofNullable(uiSettingsChanged_body)
                    .ifPresent(UiSettingsChanged_Body::close);
            Optional.ofNullable(serverSettingsChanged_body)
                    .ifPresent(ServerSettingsChanged_Body::close);
            Optional.ofNullable(playbackSettingsChanged_body)
                    .ifPresent(PlaybackSettingsChanged_Body::close);
            Optional.ofNullable(trackingSettingsChanged_body)
                    .ifPresent(TrackingSettingsChanged_Body::close);
        }
    }

    public enum Tag implements NativeMapped {
        SETTINGS_LOADED,
        SUBTITLE_SETTINGS_CHANGED,
        TORRENT_SETTINGS_CHANGED,
        UI_SETTINGS_CHANGED,
        SERVER_SETTINGS_CHANGED,
        PLAYBACK_SETTINGS_CHANGED,
        TRACKING_SETTINGS_CHANGED;

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
