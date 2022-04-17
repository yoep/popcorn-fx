package su.litvak.chromecast.api.v2;

import lombok.Builder;

import java.util.List;
import java.util.Map;

public class TestMediaStatus extends MediaStatus {
    @Builder
    public TestMediaStatus(List<Integer> activeTrackIds, long mediaSessionId, int playbackRate, PlayerState playerState, Integer currentItemId,
                           double currentTime, Map<String, Object> customData, Integer loadingItemId, List<Item> items, Integer preloadedItemId,
                           int supportedMediaCommands, Volume volume, Media media, RepeatMode repeatMode, IdleReason idleReason) {
        super(activeTrackIds, mediaSessionId, playbackRate, playerState, currentItemId, currentTime, customData, loadingItemId, items, preloadedItemId,
                supportedMediaCommands, volume, media, repeatMode, idleReason);
    }
}
