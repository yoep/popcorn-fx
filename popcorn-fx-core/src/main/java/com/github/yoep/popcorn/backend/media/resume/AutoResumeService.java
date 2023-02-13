package com.github.yoep.popcorn.backend.media.resume;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFxInstance;
import com.github.yoep.popcorn.backend.events.PlayerStoppedEvent;
import com.github.yoep.popcorn.backend.events.PlayerStoppedEventC;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.event.EventListener;
import org.springframework.stereotype.Service;

import java.util.Objects;
import java.util.Optional;

@Slf4j
@Service
@RequiredArgsConstructor
public class AutoResumeService {
    //region Getters

    /**
     * Get the resume timestamp for the given video playback.
     *
     * @param filename The filename of the video.
     * @return Returns the last known timestamp of the video if known, else {@link Optional#empty()}.
     */
    public Optional<Long> getResumeTimestamp(String filename) {
        return getResumeTimestamp(null, filename);
    }

    /**
     * Get the resume timestamp for the given video playback.
     *
     * @param id       The media ID of the video.
     * @param filename The filename of the video.
     * @return Returns the last known timestamp of the video if known, else {@link Optional#empty()}.
     */
    public Optional<Long> getResumeTimestamp(String id, String filename) {
        var ptr = FxLib.INSTANCE.auto_resume_timestamp(PopcornFxInstance.INSTANCE.get(), id, filename);
        return Optional.ofNullable(ptr)
                .map(e -> e.getLong(0));
    }

    @EventListener
    public void onClosePlayer(PlayerStoppedEvent event) {
        Objects.requireNonNull(event, "event cannot be null");
        try (var event_c = PlayerStoppedEventC.from(event)) {
            log.debug("Handling closed player event for auto-resume with {}", event_c);
            FxLib.INSTANCE.handle_player_stopped_event(PopcornFxInstance.INSTANCE.get(), event_c);
        }
    }

    //endregion
}
