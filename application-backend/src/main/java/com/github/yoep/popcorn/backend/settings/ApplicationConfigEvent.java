package com.github.yoep.popcorn.backend.settings;

import com.github.yoep.popcorn.backend.settings.models.*;
import com.sun.jna.FromNativeContext;
import com.sun.jna.NativeMapped;
import com.sun.jna.Structure;
import com.sun.jna.Union;
import lombok.Getter;
import lombok.ToString;

import java.io.Closeable;
import java.util.Arrays;

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
    }

    @Getter
    @ToString
    @FieldOrder({"settings"})
    public static class SubtitleSettingsChanged_Body extends Structure {
        public SubtitleSettings settings;
    }

    @Getter
    @ToString
    @FieldOrder({"settings"})
    public static class TorrentSettingsChanged_Body extends Structure {
        public TorrentSettings settings;
    }

    @Getter
    @ToString
    @FieldOrder({"settings"})
    public static class UiSettingsChanged_Body extends Structure {
        public UISettings settings;
    }

    @Getter
    @ToString
    @FieldOrder({"settings"})
    public static class ServerSettingsChanged_Body extends Structure {
        public ServerSettings settings;
    }

    @Getter
    @ToString
    @FieldOrder({"settings"})
    public static class PlaybackSettingsChanged_Body extends Structure {
        public PlaybackSettings settings;
    }

    @Getter
    @ToString
    public static class ApplicationConfigEventUnion extends Union {
        public static class ByValue extends ApplicationConfigEventUnion implements Union.ByValue {
        }

        public ApplicationConfigEvent.SubtitleSettingsChanged_Body subtitleSettings;
        public ApplicationConfigEvent.TorrentSettingsChanged_Body torrentSettings;
        public ApplicationConfigEvent.UiSettingsChanged_Body uiSettings;
        public ApplicationConfigEvent.ServerSettingsChanged_Body serverSettings;
        public ApplicationConfigEvent.PlaybackSettingsChanged_Body playbackSettings;
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
